use crate::context;
use crate::domain::dto::*;
use crate::domain::model::*;
use crate::error::{AppError, AppResult};
use crate::llm::{self, StreamSink};
use crate::service;
use crate::state::{AppInner, AppState};
use serde_json::json;

use std::sync::Arc;
use std::time::Instant;
use tauri::ipc::Channel;
use tauri::State;

#[tauri::command]
pub async fn chat_send(
    state: State<'_, AppState>,
    input: ChatSendInput,
    on_event: Channel<ChatEvent>,
) -> AppResult<ChatStart> {
    let inner = state.inner().inner()?;
    start_chat(inner, input, on_event, None)
}

#[tauri::command]
pub async fn chat_regenerate(
    state: State<'_, AppState>,
    input: ChatRegenerateInput,
    on_event: Channel<ChatEvent>,
) -> AppResult<ChatStart> {
    let inner = state.inner().inner()?;
    let message = inner
        .store
        .live_node(&input.message_id)
        .filter(|node| node.node_type == NodeType::Message)
        .ok_or_else(|| AppError::not_found("message"))?;
    let card_id = message.data_str("cardId").unwrap_or("").to_string();
    let turn_id = message.data_str("turnId").unwrap_or("").to_string();
    let card = inner
        .store
        .live_node(&card_id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    // The user message is the last user message in this turn, or the message itself if it is a user message.
    let user_content = if message.data_str("role") == Some("user") {
        message.body.clone()
    } else {
        let mut messages = inner
            .store
            .state()
            .read()
            .nodes
            .values()
            .filter(|node| {
                node.node_type == NodeType::Message
                    && node.is_live()
                    && node.data_str("turnId") == Some(&turn_id)
                    && node.data_str("role") == Some("user")
            })
            .cloned()
            .collect::<Vec<_>>();
        messages.sort_by_key(|node| node.created_at);
        messages.last().map(|node| node.body.clone()).unwrap_or_default()
    };
    if user_content.trim().is_empty() {
        return Err(AppError::invalid("找不到可重发的用户消息"));
    }
    let project_id = card.project_id.clone().unwrap_or_default();
    let model_profile_id = input.model_profile_id.unwrap_or_else(|| {
        inner
            .settings
            .read()
            .settings
            .model_assignments
            .chat
            .clone()
            .unwrap_or_default()
    });
    let send = ChatSendInput {
        project_id,
        card_id,
        content: user_content,
        model_profile_id,
        attachments: vec![],
        context: ContextOptions::default(),
    };
    start_chat(inner, send, on_event, Some(message.id))
}

#[tauri::command]
pub fn chat_cancel(state: State<'_, AppState>, run_id: String) -> AppResult<()> {
    state.inner().inner()?.request_cancel(&run_id);
    Ok(())
}

fn start_chat(
    inner: Arc<AppInner>,
    input: ChatSendInput,
    on_event: Channel<ChatEvent>,
    regenerated_from: Option<String>,
) -> AppResult<ChatStart> {
    let project = service::project_node(&inner.store, &input.project_id)?;
    if project.data_bool("readOnly").unwrap_or(false) {
        return Err(AppError::new("invalid_input", "项目只读，无法发送"));
    }
    let card = inner
        .store
        .live_node(&input.card_id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    if card.locked {
        return Err(AppError::new("invalid_input", "卡片已锁定，无法发送"));
    }
    let (profile_node, model) = inner.model_profile(&input.model_profile_id)?;
    let key = if model.provider == "mock" {
        None
    } else {
        let key = inner.api_key(&profile_node.id)?;
        if key.is_none() {
            return Err(AppError::new("key_missing", "这个模型还没有设置 API Key"));
        }
        key
    };
    if input.content.trim().is_empty() {
        return Err(AppError::invalid("消息不能为空"));
    }

    let compiled = context::compile(
        &inner.store,
        &input.card_id,
        &input.content,
        &model,
        &input.context,
    )?;

    let turn_index = inner
        .store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Turn
                && node.is_live()
                && node.data_str("cardId") == Some(&input.card_id)
        })
        .map(|node| node.data_i64("index").unwrap_or(0))
        .max()
        .unwrap_or(-1)
        + 1;

    let turn = Node::new(
        NodeType::Turn,
        Some(input.project_id.clone()),
        format!("Round {}", turn_index + 1),
        "",
        json!({
            "cardId": input.card_id,
            "index": turn_index,
            "collapsed": false,
            "modelProfileId": input.model_profile_id,
            "contextSnapshotId": null,
        }),
    );
    let user_message = Node::new(
        NodeType::Message,
        Some(input.project_id.clone()),
        "You",
        input.content.clone(),
        json!({
            "role": "user",
            "status": "complete",
            "reasoning": "",
            "citations": [],
            "attachments": [],
            "turnId": turn.id,
            "cardId": input.card_id,
            "usage": null,
            "index": 0,
        }),
    );
    let assistant_message = Node::new(
        NodeType::Message,
        Some(input.project_id.clone()),
        "Sporelink",
        "",
        json!({
            "role": "assistant",
            "status": "streaming",
            "reasoning": "",
            "citations": [],
            "attachments": [],
            "turnId": turn.id,
            "cardId": input.card_id,
            "usage": null,
            "regeneratedFrom": regenerated_from,
            "index": 1,
        }),
    );

    let mut edges = vec![
        Edge::new(
            Some(input.project_id.clone()),
            &input.card_id,
            &turn.id,
            EdgeRelation::Contains,
            turn_index,
            json!({}),
        ),
        Edge::new(
            Some(input.project_id.clone()),
            &turn.id,
            &user_message.id,
            EdgeRelation::Contains,
            0,
            json!({}),
        ),
        Edge::new(
            Some(input.project_id.clone()),
            &turn.id,
            &assistant_message.id,
            EdgeRelation::Contains,
            1,
            json!({}),
        ),
    ];
    if let Some(original) = &regenerated_from {
        edges.push(Edge::new(
            Some(input.project_id.clone()),
            &assistant_message.id,
            original,
            EdgeRelation::References,
            0,
            json!({ "regeneratedFrom": original }),
        ));
    }
    inner.store.put_batch(
        vec![turn.clone(), user_message.clone(), assistant_message.clone()],
        edges,
    )?;

    let snapshot = Node::new(
        NodeType::ContextSnapshot,
        Some(input.project_id.clone()),
        "Context snapshot",
        "",
        json!({
            "cardId": input.card_id,
            "turnId": turn.id,
            "modelProfileId": input.model_profile_id,
            "items": compiled.items.clone(),
            "totalTokens": compiled.preview.total_tokens,
            "budget": compiled.preview.budget,
        }),
    );
    let snapshot = inner.store.put_node(snapshot, None)?;
    let mut turn = inner.store.live_node(&turn.id).ok_or_else(|| AppError::not_found("turn"))?;
    if let Some(data) = turn.data.as_object_mut() {
        data.insert("contextSnapshotId".into(), json!(snapshot.id));
    }
    inner.store.put_node(turn, None)?;

    let run_id = new_id();
    let cancel = inner.cancel_token(&run_id);
    let start = ChatStart {
        run_id: run_id.clone(),
        turn_id: snapshot
            .data_str("turnId")
            .unwrap_or_default()
            .to_string(),
        user_message_id: user_message.id.clone(),
        assistant_message_id: assistant_message.id.clone(),
        context_snapshot_id: snapshot.id.clone(),
    };

    let run = Node::new(
        NodeType::Run,
        Some(input.project_id.clone()),
        "Run",
        "",
        json!({
            "provider": model.provider,
            "model": model.model,
            "purpose": "chat",
            "status": "streaming",
            "inputTokens": 0,
            "outputTokens": 0,
            "cost": 0.0,
            "latencyMs": 0,
            "error": null,
            "contextSnapshotId": snapshot.id,
        }),
    );
    let run = inner.store.put_node(run, None)?;

    let messages = compiled.messages;
    let channel = on_event.clone();
    let inner_task = inner.clone();
    let assistant_id = assistant_message.id.clone();
    let user_message_id = user_message.id.clone();
    let run_id_task = run_id.clone();
    let run_node_id = run.id.clone();
    let checkpoint_ms = inner
        .settings
        .read()
        .settings
        .auto_behavior
        .checkpoint_ms
        .max(150);
    let started = start.clone();
    tauri::async_runtime::spawn(async move {
        let _ = channel.send(ChatEvent::Started {
            run_id: started.run_id.clone(),
            turn_id: started.turn_id.clone(),
            user_message_id,
            assistant_message_id: assistant_id.clone(),
            context_snapshot_id: started.context_snapshot_id.clone(),
        });
        let text = Arc::new(parking_lot::Mutex::new(String::new()));
        let reasoning = Arc::new(parking_lot::Mutex::new(String::new()));
        let last_checkpoint = Arc::new(parking_lot::Mutex::new(Instant::now()));
        let channel_delta = channel.clone();
        let channel_reasoning = channel.clone();
        let run_id_delta = run_id_task.clone();
        let assistant_delta = assistant_id.clone();
        let assistant_reasoning = assistant_id.clone();
        let store_delta = inner_task.store.clone();
        let checkpoint_ms_delta = checkpoint_ms;
        let text_delta = text.clone();
        let reasoning_delta = reasoning.clone();
        let last_delta = last_checkpoint.clone();

        let mut sink = StreamSink {
            on_delta: &mut |delta: String| {
                text_delta.lock().push_str(&delta);
                let _ = channel_delta.send(ChatEvent::Delta {
                    run_id: run_id_delta.clone(),
                    message_id: assistant_delta.clone(),
                    text: delta,
                });
                let mut last = last_delta.lock();
                if last.elapsed().as_millis() as u64 >= checkpoint_ms_delta {
                    *last = Instant::now();
                    let text = text_delta.lock().clone();
                    let reasoning = reasoning_delta.lock().clone();
                    let _ = save_assistant(&store_delta, &assistant_delta, &text, &reasoning, "streaming", None);
                }
            },
            on_reasoning: &mut |delta: String| {
                reasoning_delta.lock().push_str(&delta);
                let _ = channel_reasoning.send(ChatEvent::Reasoning {
                    run_id: run_id_task.clone(),
                    message_id: assistant_reasoning.clone(),
                    text: delta,
                });
            },
            on_usage: &mut |_input: u64, _output: u64| {},
        };

        let result = llm::stream_chat(
            &inner_task.http,
            &model,
            key.as_deref(),
            &messages,
            cancel,
            &mut sink,
        )
        .await;

        let (status, outcome_text, outcome_reasoning, usage, error) = match result {
            Ok(outcome) => {
                let status = if outcome.cancelled { "cancelled" } else { "complete" };
                let usage = Usage {
                    input_tokens: outcome.input_tokens,
                    output_tokens: outcome.output_tokens,
                    cost: outcome.cost,
                    currency: outcome.currency.clone(),
                    latency_ms: outcome.latency_ms,
                    estimated: false,
                };
                (status, outcome.text, outcome.reasoning, Some(usage), None)
            }
            Err(error) => {
                let text = text.lock().clone();
                let reasoning = reasoning.lock().clone();
                (
                    "error",
                    text,
                    reasoning,
                    None,
                    Some((error.code.clone(), error.message.clone(), error.hint.clone(), error.retryable)),
                )
            }
        };
        let _ = save_assistant(
            &inner_task.store,
            &assistant_id,
            &outcome_text,
            &outcome_reasoning,
            status,
            usage.clone(),
        );
        if let Some(run_node) = inner_task.store.live_node(&run_node_id) {
            let mut run_node = run_node;
            if let Some(data) = run_node.data.as_object_mut() {
                data.insert("status".into(), json!(status));
                data.insert("inputTokens".into(), json!(usage.as_ref().map(|u| u.input_tokens).unwrap_or(0)));
                data.insert("outputTokens".into(), json!(usage.as_ref().map(|u| u.output_tokens).unwrap_or(0)));
                data.insert("cost".into(), json!(usage.as_ref().map(|u| u.cost).unwrap_or(0.0)));
                data.insert("latencyMs".into(), json!(usage.as_ref().map(|u| u.latency_ms).unwrap_or(0)));
                data.insert("error".into(), json!(error.as_ref().map(|e| e.1.clone())));
            }
            let _ = inner_task.store.put_node(run_node, None);
        }
        if let Some(usage) = usage {
            let _ = channel.send(ChatEvent::Usage {
                run_id: run_id_task.clone(),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cost: usage.cost,
                currency: usage.currency,
                latency_ms: usage.latency_ms,
                estimated: usage.estimated,
            });
        }
        match status {
            "complete" => {
                let _ = channel.send(ChatEvent::Done {
                    run_id: run_id_task.clone(),
                    message_id: assistant_id,
                    status: "complete".into(),
                });
            }
            "cancelled" => {
                let _ = channel.send(ChatEvent::Cancelled {
                    run_id: run_id_task.clone(),
                    message_id: assistant_id,
                });
            }
            _ => {
                if let Some((code, message, hint, retryable)) = error {
                    let _ = channel.send(ChatEvent::Error {
                        run_id: run_id_task.clone(),
                        code,
                        message,
                        hint,
                        retryable,
                    });
                }
            }
        }
        inner_task.clear_cancel(&run_id_task);
    });

    Ok(start)
}

fn save_assistant(
    store: &crate::store::Store,
    id: &str,
    text: &str,
    reasoning: &str,
    status: &str,
    usage: Option<Usage>,
) -> AppResult<()> {
    let mut node = store
        .live_node(id)
        .ok_or_else(|| AppError::not_found("assistant message"))?;
    node.body = text.to_string();
    if let Some(data) = node.data.as_object_mut() {
        data.insert("reasoning".into(), json!(reasoning));
        data.insert("status".into(), json!(status));
        data.insert("usage".into(), json!(usage));
    }
    store.put_node(node, None)?;
    Ok(())
}
