use crate::domain::dto::{
    ContextItem, ContextOptions, ContextPreview, MessageDto, ModelProfileData,
};
use crate::domain::model::{EdgeRelation, Node, NodeType};
use crate::error::{AppError, AppResult};
use crate::store::Store;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct CompiledContext {
    pub messages: Vec<ChatMessage>,
    pub preview: ContextPreview,
    pub items: Vec<ContextItem>,
    pub system_prompt: String,
}

pub fn estimate_tokens(text: &str) -> u64 {
    let mut cjk = 0u64;
    let mut other = 0u64;
    for ch in text.chars() {
        let cp = ch as u32;
        if (0x3400..=0x9fff).contains(&cp)
            || (0xf900..=0xfaff).contains(&cp)
            || (0x3040..=0x30ff).contains(&cp)
            || (0xac00..=0xd7af).contains(&cp)
        {
            cjk += 1;
        } else {
            other += 1;
        }
    }
    (cjk + other.div_ceil(4)).max(1)
}

fn truncate_to_tokens(text: &str, tokens: u64) -> String {
    if estimate_tokens(text) <= tokens {
        return text.to_string();
    }
    let mut out = String::new();
    let mut count = 0.0f64;
    let limit = tokens.max(1) as f64;
    for ch in text.chars() {
        let cp = ch as u32;
        let cost = if (0x3400..=0x9fff).contains(&cp)
            || (0xf900..=0xfaff).contains(&cp)
            || (0x3040..=0x30ff).contains(&cp)
            || (0xac00..=0xd7af).contains(&cp)
        {
            1.0
        } else {
            0.25
        };
        if count + cost > limit {
            break;
        }
        count += cost;
        out.push(ch);
    }
    if out.is_empty() {
        out = text.chars().take(8).collect();
    }
    out
}

fn card_path(store: &Store, card_id: &str) -> AppResult<Vec<Node>> {
    let mut path = Vec::new();
    let mut current = store
        .live_node(card_id)
        .ok_or_else(|| AppError::not_found("card"))?;
    let mut guard = 0;
    loop {
        path.push(current.clone());
        if guard > 200 {
            return Err(AppError::invalid("卡片树深度异常"));
        }
        guard += 1;
        let parent_edge = store
            .incoming_edges(&current.id)
            .into_iter()
            .find(|edge| edge.relation == EdgeRelation::Contains)
            .filter(|edge| {
                store
                    .live_node(&edge.from)
                    .map(|n| n.node_type == NodeType::Card)
                    .unwrap_or(false)
            });
        match parent_edge {
            Some(edge) => {
                current = store
                    .live_node(&edge.from)
                    .ok_or_else(|| AppError::not_found("parent card"))?;
            }
            None => break,
        }
    }
    path.reverse();
    Ok(path)
}

fn turns_for_card(store: &Store, card_id: &str) -> Vec<Node> {
    let mut turns = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Turn
                && node.is_live()
                && node.data_str("cardId") == Some(card_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    turns.sort_by_key(|turn| turn.data_i64("index").unwrap_or(0));
    turns
}

fn messages_for_turn(store: &Store, turn_id: &str) -> Vec<Node> {
    let mut messages = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Message
                && node.is_live()
                && node.data_str("turnId") == Some(turn_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| {
        (
            message.data_i64("index").unwrap_or(i64::MAX),
            message.created_at,
        )
    });
    messages
}

fn messages_for_card(store: &Store, card_id: &str) -> Vec<Node> {
    let mut messages = Vec::new();
    for turn in turns_for_card(store, card_id) {
        messages.extend(messages_for_turn(store, &turn.id));
    }
    messages
}

fn message_role(message: &Node) -> String {
    message
        .data_str("role")
        .unwrap_or("assistant")
        .to_string()
}

fn first_line(text: &str, max: usize) -> String {
    let line = text.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
    let mut out = String::new();
    for ch in line.trim().chars().take(max) {
        out.push(ch);
    }
    if line.trim().chars().count() > max {
        out.push('…');
    }
    if out.is_empty() {
        "(empty)".into()
    } else {
        out
    }
}

fn make_item(
    id: String,
    kind: &str,
    source_id: &str,
    title: String,
    text: &str,
    reason: &str,
) -> ContextItem {
    ContextItem {
        id,
        kind: kind.into(),
        source_id: source_id.into(),
        title,
        tokens: estimate_tokens(text),
        reason: reason.into(),
        included: false,
        dropped_reason: None,
        path: None,
    }
}

pub fn compile(
    store: &Store,
    card_id: &str,
    draft: &str,
    model: &ModelProfileData,
    options: &ContextOptions,
    locale: &str,
) -> AppResult<CompiledContext> {
    let path = card_path(store, card_id)?;
    let current = path.last().ok_or_else(|| AppError::not_found("card"))?;

    // Cut point from the current card's branch/deep-dive edge.
    let mut cut_message_id: Option<String> = None;
    for edge in store.outgoing_edges(&current.id) {
        if edge.relation == EdgeRelation::BranchOf || edge.relation == EdgeRelation::DerivesFrom {
            if let Some(value) = edge
                .metadata
                .get("contextCutMessageId")
                .or_else(|| edge.metadata.get("sourceMessageId"))
                .and_then(|v| v.as_str())
            {
                cut_message_id = Some(value.to_string());
            }
        }
    }

    let mut history: Vec<Node> = Vec::new();
    for card in path.iter().take(path.len().saturating_sub(1)) {
        history.extend(messages_for_card(store, &card.id));
    }
    if let Some(cut) = &cut_message_id {
        if let Some(index) = history.iter().position(|m| &m.id == cut) {
            history.truncate(index + 1);
        }
    }
    let current_messages = messages_for_card(store, &current.id);
    history.extend(current_messages);

    let budget = (model.context_window.saturating_sub(model.max_output)).max(1024) as u64;
    let mut items: Vec<ContextItem> = Vec::new();

    let base_prompt = system_prompt(model, locale);
    let mut system_tokens = estimate_tokens(&base_prompt);
    let draft_text = truncate_to_tokens(draft, budget / 2);
    let draft_item = make_item(
        "draft".into(),
        "path",
        "draft",
        "Current input".into(),
        &draft_text,
        "最后一条用户消息，始终保留",
    );
    let system_item = ContextItem {
        id: "system".into(),
        kind: "system".into(),
        source_id: "system".into(),
        title: "System prompt".into(),
        tokens: system_tokens,
        reason: "模型角色、语言与本地优先约束".into(),
        included: true,
        dropped_reason: None,
        path: None,
    };
    items.push(system_item);

    for message in &history {
        let role = message_role(message);
        let title = format!(
            "{}: {}",
            if role == "user" { "You" } else { "Assistant" },
            first_line(&message.body, 40)
        );
        items.push(make_item(
            format!("message:{}", message.id),
            "path",
            &message.id,
            title,
            &message.body,
            "当前卡片路径上的消息",
        ));
    }

    let mut pinned = Vec::new();
    for id in &options.pinned_message_ids {
        if let Some(message) = store.live_node(id) {
            items.push(make_item(
                format!("pinned:{}", message.id),
                "pinned",
                &message.id,
                format!("Pinned: {}", first_line(&message.body, 40)),
                &message.body,
                "用户置顶消息",
            ));
            pinned.push(message);
        }
    }

    let mut knowledge = Vec::new();
    for id in &options.knowledge_ids {
        if let Some(node) = store.live_node(id) {
            if node.node_type == NodeType::Knowledge {
                knowledge.push(node.clone());
                items.push(make_item(
                    format!("knowledge:{}", node.id),
                    "knowledge",
                    &node.id,
                    node.title.clone(),
                    &node.body,
                    "用户选中的知识条目",
                ));
            }
        }
    }

    let mut chunks = Vec::new();
    for id in &options.document_chunk_ids {
        if let Some(node) = store.live_node(id) {
            if node.node_type == NodeType::Chunk {
                let path = node.data_str("path").unwrap_or("").to_string();
                let locator = node
                    .data_i64("page")
                    .map(|page| format!("p{page}"))
                    .or_else(|| node.data_str("heading").map(str::to_string))
                    .unwrap_or_else(|| format!("#{}", node.data_i64("index").unwrap_or(0) + 1));
                let reference = if path.is_empty() {
                    format!("workspace://chunk/{}", node.id)
                } else {
                    format!("{path}#{locator}")
                };
                let title = reference.clone();
                chunks.push((node.clone(), reference.clone()));
                items.push(ContextItem {
                    id: format!("document:{}", node.id),
                    kind: "document".into(),
                    source_id: node.id.clone(),
                    title,
                    tokens: 16,
                    reason: "用户选中的工作区文件引用".into(),
                    included: false,
                    dropped_reason: None,
                    path: if path.is_empty() { None } else { Some(path) },
                });
            }
        }
    }

    // The draft is always included; system is always included.
    let mut included: HashSet<String> = HashSet::new();
    included.insert("system".into());
    included.insert("draft".into());
    let mut total = system_tokens + draft_item.tokens;

    // Pinned first, then recent path messages from newest to oldest.
    for item in items.iter().filter(|i| i.kind == "pinned") {
        if !options.excluded_item_ids.contains(&item.id) && total + item.tokens <= budget {
            total += item.tokens;
            included.insert(item.id.clone());
        }
    }
    let mut path_items = items.iter().filter(|i| i.kind == "path" && i.id != "draft").collect::<Vec<_>>();
    path_items.reverse();
    for item in path_items {
        if !options.excluded_item_ids.contains(&item.id) && total + item.tokens <= budget {
            total += item.tokens;
            included.insert(item.id.clone());
        }
    }
    for kind in ["knowledge", "document"] {
        for item in items.iter().filter(|i| i.kind == kind) {
            if !options.excluded_item_ids.contains(&item.id) && total + item.tokens <= budget {
                total += item.tokens;
                included.insert(item.id.clone());
            }
        }
    }

    let mut dropped_count = 0;
    for item in items.iter_mut() {
        if item.id == "system" || item.id == "draft" {
            item.included = true;
            continue;
        }
        if options.excluded_item_ids.contains(&item.id) {
            item.included = false;
            item.dropped_reason = Some("用户禁用".into());
            dropped_count += 1;
            continue;
        }
        if included.contains(&item.id) {
            item.included = true;
        } else {
            item.included = false;
            item.dropped_reason = Some("超出上下文预算".into());
            dropped_count += 1;
        }
    }

    // Build system message with reference material.
    let base_system_tokens = system_tokens;
    let is_included = |id: &str| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.included)
            .unwrap_or(false)
    };
    let mut system = base_prompt.clone();
    if pinned.iter().any(|node| is_included(&format!("pinned:{}", node.id))) {
        system.push_str("\n\n## Pinned context\n");
        for node in &pinned {
            if is_included(&format!("pinned:{}", node.id)) {
                system.push_str(&format!("\n### Pinned message\n{}\n", node.body));
            }
        }
    }
    if knowledge
        .iter()
        .any(|node| is_included(&format!("knowledge:{}", node.id)))
    {
        system.push_str("\n\n## Selected knowledge\n");
        for node in &knowledge {
            if is_included(&format!("knowledge:{}", node.id)) {
                system.push_str(&format!("\n### {}\n{}\n", node.title, node.body));
            }
        }
    }
    if chunks
        .iter()
        .any(|(node, _)| is_included(&format!("document:{}", node.id)))
    {
        system.push_str("\n\n## Selected workspace file references\n");
        system.push_str(
            "These are workspace paths and locators, not file contents. Use the read-only workspace_read_file tool with a path and optional line range to read them.\n",
        );
        for (node, reference) in &chunks {
            if is_included(&format!("document:{}", node.id)) {
                system.push_str(&format!("\n- {reference}"));
            }
        }
        system.push('\n');
    }
    system_tokens = estimate_tokens(&system);
    total = total
        .saturating_sub(base_system_tokens)
        .saturating_add(system_tokens);
    if let Some(item) = items.iter_mut().find(|item| item.id == "system") {
        item.tokens = system_tokens;
    }

    let mut messages = vec![ChatMessage {
        role: "system".into(),
        content: system.clone(),
    }];
    for item in &items {
        if item.kind == "path" && item.id != "draft" && item.included {
            if let Some(source_id) = item.source_id.strip_prefix("draft") {
                let _ = source_id;
                continue;
            }
            if let Some(message) = store.live_node(&item.source_id) {
                messages.push(ChatMessage {
                    role: message_role(&message),
                    content: message.body.clone(),
                });
            }
        }
    }
    if !draft_text.trim().is_empty() {
        messages.push(ChatMessage {
            role: "user".into(),
            content: draft_text,
        });
    }

    let preview = ContextPreview {
        items: items.clone(),
        total_tokens: total,
        budget,
        dropped_count,
        model_profile_id: String::new(),
        message_count: history.len() + 1,
        system_prompt_tokens: system_tokens,
    };

    Ok(CompiledContext {
        messages,
        preview,
        items,
        system_prompt: system,
    })
}

fn system_prompt(model: &ModelProfileData, locale: &str) -> String {
    let language = match locale {
        "en" => "Reply in English unless the user asks for another language.",
        "zh-TW" => "預設使用繁體中文回覆，除非使用者要求其他語言。",
        _ => "默认使用简体中文回复，除非用户要求其他语言。",
    };
    format!(
        "You are Sporelink, a local-first knowledge exploration assistant. \
         The user's notes and conversations are stored locally on their machine. \
         Be precise, cite the provided reference material when relevant, and say when something is uncertain. \
         {language}\n\n\
         Model: {} ({})\n\
         Use Markdown; use KaTeX for math and fenced code blocks for code.",
        model.model, model.provider
    )
}

pub fn preview_dto(compiled: &CompiledContext, model_profile_id: &str) -> ContextPreview {
    let mut preview = compiled.preview.clone();
    preview.model_profile_id = model_profile_id.into();
    preview
}

pub fn messages_json(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|message| {
            serde_json::json!({
                "role": message.role,
                "content": message.content,
            })
        })
        .collect()
}

pub fn message_to_dto(store: &Store, message: &Node) -> MessageDto {
    let mut dto = MessageDto {
        id: message.id.clone(),
        turn_id: message.data_str("turnId").unwrap_or("").into(),
        card_id: message.data_str("cardId").unwrap_or("").into(),
        role: message_role(message),
        content: message.body.clone(),
        reasoning: message.data_str("reasoning").unwrap_or("").into(),
        citations: serde_json::from_value(
            message
                .data
                .get("citations")
                .cloned()
                .unwrap_or(serde_json::json!([])),
        )
        .unwrap_or_default(),
        attachments: serde_json::from_value(
            message
                .data
                .get("attachments")
                .cloned()
                .unwrap_or(serde_json::json!([])),
        )
        .unwrap_or_default(),
        status: message.data_str("status").unwrap_or("complete").into(),
        favorite: message.favorite,
        edited_from: message.data_str("editedFrom").map(|s| s.to_string()),
        regenerated_from: message.data_str("regeneratedFrom").map(|s| s.to_string()),
        usage: serde_json::from_value(
            message
                .data
                .get("usage")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .ok(),
        created_at: message.created_at,
        updated_at: message.updated_at,
        revision: message.revision,
    };
    if dto.turn_id.is_empty() {
        if let Some(turn) = store
            .incoming_edges(&message.id)
            .into_iter()
            .find(|edge| edge.relation == EdgeRelation::Contains)
        {
            dto.turn_id = turn.from;
        }
    }
    if dto.card_id.is_empty() && !dto.turn_id.is_empty() {
        if let Some(turn) = store.node(&dto.turn_id) {
            dto.card_id = turn.data_str("cardId").unwrap_or("").into();
        }
    }
    dto
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_cjk_and_latin() {
        assert!(estimate_tokens("你好世界") >= 4);
        assert!(estimate_tokens("hello world") >= 2);
    }

    #[test]
    fn truncation_keeps_budget() {
        let text = "hello world ".repeat(500);
        let out = truncate_to_tokens(&text, 12);
        assert!(estimate_tokens(&out) <= 14);
        assert!(!out.is_empty());
    }
}
