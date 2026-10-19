use crate::context::{estimate_tokens, ChatMessage};
use crate::domain::dto::{ModelProfileData, ModelTestResult};
use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct StreamSink<'a> {
    pub on_delta: &'a mut (dyn FnMut(String) + Send),
    pub on_reasoning: &'a mut (dyn FnMut(String) + Send),
    pub on_usage: &'a mut (dyn FnMut(u64, u64) + Send),
}

#[derive(Clone, Debug, Default)]
pub struct LlmOutcome {
    pub text: String,
    pub reasoning: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub currency: String,
    pub latency_ms: u64,
    pub cancelled: bool,
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(20))
        .user_agent("Sporelink/0.1.0-alpha")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn provider_error(status: reqwest::StatusCode, body: &str) -> AppError {
    let code = if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        "provider_auth"
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "provider_rate_limit"
    } else if status.is_server_error() {
        "provider_unavailable"
    } else {
        "provider_unavailable"
    };
    let snippet = body.chars().take(500).collect::<String>();
    AppError::new(code, format!("供应商返回 {status}: {snippet}"))
        .retryable(status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS)
}

fn request_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::new("provider_timeout", "模型请求超时").retryable(true)
    } else {
        AppError::new("provider_unavailable", format!("网络请求失败：{error}")).retryable(true)
    }
}

fn cost_for(model: &ModelProfileData, input: u64, output: u64) -> f64 {
    (input as f64 / 1_000_000.0) * model.pricing.input_per_million
        + (output as f64 / 1_000_000.0) * model.pricing.output_per_million
}

pub async fn stream_chat(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    sink: &mut StreamSink<'_>,
) -> AppResult<LlmOutcome> {
    let started = Instant::now();
    let mut outcome = match model.provider.as_str() {
        "mock" => mock_stream(model, messages, cancel.clone(), sink).await?,
        "openai_compatible" => openai_stream(client, model, api_key, messages, cancel.clone(), sink).await?,
        "anthropic" => anthropic_stream(client, model, api_key, messages, cancel.clone(), sink).await?,
        "gemini" => gemini_stream(client, model, api_key, messages, cancel.clone(), sink).await?,
        other => return Err(AppError::unsupported(format!("未知供应商：{other}"))),
    };
    outcome.latency_ms = started.elapsed().as_millis() as u64;
    if outcome.input_tokens == 0 {
        outcome.input_tokens = messages.iter().map(|m| estimate_tokens(&m.content)).sum();
    }
    if outcome.output_tokens == 0 {
        outcome.output_tokens = estimate_tokens(&outcome.text) + estimate_tokens(&outcome.reasoning);
    }
    outcome.cost = cost_for(model, outcome.input_tokens, outcome.output_tokens);
    outcome.currency = if model.pricing.currency.is_empty() {
        "USD".into()
    } else {
        model.pricing.currency.clone()
    };
    (sink.on_usage)(outcome.input_tokens, outcome.output_tokens);
    Ok(outcome)
}

pub async fn complete(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    purpose: &str,
) -> AppResult<LlmOutcome> {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut sink = StreamSink {
        on_delta: &mut |delta: String| text.push_str(&delta),
        on_reasoning: &mut |delta: String| reasoning.push_str(&delta),
        on_usage: &mut |_i: u64, _o: u64| {},
    };
    let mut messages = messages.to_vec();
    messages.insert(
        0,
        ChatMessage {
            role: "system".into(),
            content: format!("TASK:{purpose}\nYou are producing a durable knowledge artifact for a local-first app. Be concise and structured."),
        },
    );
    stream_chat(client, model, api_key, &messages, cancel, &mut sink).await
}

// ---------------------------------------------------------------------------
// Mock
// ---------------------------------------------------------------------------

async fn mock_stream(
    _model: &ModelProfileData,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    sink: &mut StreamSink<'_>,
) -> AppResult<LlmOutcome> {
    let last_user = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();
    let system = messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let is_summary = system.contains("TASK:summary");
    let is_check = system.contains("TASK:knowledge_check");
    let is_title = system.contains("TASK:title");

    let reasoning = if is_summary || is_check || is_title {
        String::new()
    } else {
        format!(
            "我先确认用户的目标：{}。\n然后给出一个可操作的框架：概念、例子、边界、下一步。",
            last_user.chars().take(120).collect::<String>()
        )
    };
    let text = if is_summary {
        format!(
            "## 项目摘要\n\n本次探索围绕以下问题展开：\n\n> {}\n\n### 主要线索\n\n- 建立了卡片树，区分深潜、发散与分支。\n- 使用上下文预览确认发送内容。\n- 把关键理解保存为知识条目。\n\n### 可以继续的方向\n\n1. 为每个分支补一个可验证的例子。\n2. 把结论与来源消息关联。\n3. 导出为 Markdown 后做一次复盘。\n",
            last_user.chars().take(160).collect::<String>()
        )
    } else if is_check {
        format!(
            "你的解释抓住了主要方向。可以补强两点：\n\n1. 给核心概念一个具体例子，说明它在什么条件下成立。\n2. 指出一个反例或边界，避免把结论绝对化。\n\n整体可以作为一条可引用的知识条目保留。\n\n原始输入：\n> {}\n",
            last_user.chars().take(300).collect::<String>()
        )
    } else if is_title {
        format!("关于「{}」的探索", last_user.chars().take(24).collect::<String>())
    } else {
        format!(
            "## 先给一个可操作的框架\n\n你问的是：\n\n> {}\n\n### 1. 先把问题拆开\n\n- 这个概念解决什么问题？\n- 它在什么条件下成立？\n- 有哪些常见的反例或边界？\n\n### 2. 一个具体例子\n\n假设你要向一个刚接触这个领域的人解释它。先用一句话给出直觉，再给一个最小例子，最后说明这个例子为什么能支持你的结论。\n\n```text\n直觉 → 最小例子 → 边界条件 → 可验证的下一步\n```\n\n### 3. 一个可验证的下一步\n\n把上面的框架写成三条可检验的陈述，然后逐条找证据。数学上，像 $e^{{i\\pi}} + 1 = 0$ 这样的式子提醒我们：漂亮的统一形式背后，仍然需要定义和条件。\n\n### 4. 记到哪里\n\n把其中最有价值的一条选中，保存为知识条目；然后用“深潜”继续追问细节，或用“分支”尝试一个相反的解释。\n\n---\n\n这是 Mock 模型的离线回复。它不会联网；配置真实供应商后，同一套卡片、上下文与导出流程可以直接复用。",
            last_user.chars().take(160).collect::<String>()
        )
    };

    for ch in reasoning.chars() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(LlmOutcome {
                text,
                reasoning,
                cancelled: true,
                ..Default::default()
            });
        }
        (sink.on_reasoning)(ch.to_string());
        sleep(Duration::from_millis(2)).await;
    }
    let chunks = text.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < chunks.len() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(LlmOutcome {
                text,
                reasoning,
                cancelled: true,
                ..Default::default()
            });
        }
        let take = if index + 6 < chunks.len() { 6 } else { chunks.len() - index };
        let piece = chunks[index..index + take].iter().collect::<String>();
        (sink.on_delta)(piece);
        index += take;
        sleep(Duration::from_millis(5)).await;
    }
    let input_tokens = messages.iter().map(|m| estimate_tokens(&m.content)).sum();
    let output_tokens = estimate_tokens(&text) + estimate_tokens(&reasoning);
    Ok(LlmOutcome {
        text,
        reasoning,
        input_tokens,
        output_tokens,
        cost: 0.0,
        currency: "USD".into(),
        latency_ms: 0,
        cancelled: false,
    })
}

// ---------------------------------------------------------------------------
// OpenAI-compatible
// ---------------------------------------------------------------------------

fn openai_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

fn openai_models_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/v1") {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    }
}

async fn openai_stream(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    sink: &mut StreamSink<'_>,
) -> AppResult<LlmOutcome> {
    let key = api_key.ok_or_else(|| AppError::new("key_missing", "这个模型还没有设置 API Key"))?;
    let body = json!({
        "model": model.model,
        "messages": messages.iter().map(|m| json!({ "role": m.role, "content": m.content })).collect::<Vec<_>>(),
        "stream": true,
        "stream_options": { "include_usage": true },
        "max_tokens": model.max_output,
    });
    let response = client
        .post(openai_url(&model.base_url))
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(request_error)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(provider_error(status, &body));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Ok(LlmOutcome {
                text,
                reasoning,
                input_tokens,
                output_tokens,
                cancelled: true,
                ..Default::default()
            });
        }
        let chunk = chunk.map_err(request_error)?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(value) = serde_json::from_str::<Value>(data) {
                    if let Some(usage) = value.get("usage") {
                        input_tokens = usage
                            .get("prompt_tokens")
                            .and_then(Value::as_u64)
                            .unwrap_or(input_tokens);
                        output_tokens = usage
                            .get("completion_tokens")
                            .and_then(Value::as_u64)
                            .unwrap_or(output_tokens);
                    }
                    if let Some(delta) = value
                        .get("choices")
                        .and_then(Value::as_array)
                        .and_then(|a| a.first())
                        .and_then(|c| c.get("delta"))
                    {
                        if let Some(content) = delta.get("content").and_then(Value::as_str) {
                            if !content.is_empty() {
                                text.push_str(content);
                                (sink.on_delta)(content.to_string());
                            }
                        }
                        let reason = delta
                            .get("reasoning_content")
                            .or_else(|| delta.get("reasoning"))
                            .and_then(Value::as_str);
                        if let Some(reason) = reason {
                            if !reason.is_empty() {
                                reasoning.push_str(reason);
                                (sink.on_reasoning)(reason.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(LlmOutcome {
        text,
        reasoning,
        input_tokens,
        output_tokens,
        cancelled: false,
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Anthropic
// ---------------------------------------------------------------------------

fn anthropic_url(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/v1/messages") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/messages")
    } else {
        format!("{base}/v1/messages")
    }
}

async fn anthropic_stream(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    sink: &mut StreamSink<'_>,
) -> AppResult<LlmOutcome> {
    let key = api_key.ok_or_else(|| AppError::new("key_missing", "这个模型还没有设置 API Key"))?;
    let system = messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let api_messages = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| json!({ "role": if m.role == "assistant" { "assistant" } else { "user" }, "content": m.content }))
        .collect::<Vec<_>>();
    let body = json!({
        "model": model.model,
        "max_tokens": model.max_output,
        "stream": true,
        "system": system,
        "messages": api_messages,
    });
    let response = client
        .post(anthropic_url(&model.base_url))
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(request_error)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(provider_error(status, &body));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Ok(LlmOutcome {
                text,
                reasoning,
                input_tokens,
                output_tokens,
                cancelled: true,
                ..Default::default()
            });
        }
        let chunk = chunk.map_err(request_error)?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line.trim_start_matches("data:").trim();
            if data.is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(data) {
                if let Some(message) = value.get("message") {
                    if let Some(usage) = message.get("usage") {
                        input_tokens = usage
                            .get("input_tokens")
                            .and_then(Value::as_u64)
                            .unwrap_or(input_tokens);
                    }
                }
                if let Some(usage) = value.get("usage") {
                    input_tokens = usage
                        .get("input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(input_tokens);
                    output_tokens = usage
                        .get("output_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(output_tokens);
                }
                if let Some(delta) = value.get("delta") {
                    if let Some(content) = delta.get("text").and_then(Value::as_str) {
                        text.push_str(content);
                        (sink.on_delta)(content.to_string());
                    }
                    if let Some(thinking) = delta.get("thinking").and_then(Value::as_str) {
                        reasoning.push_str(thinking);
                        (sink.on_reasoning)(thinking.to_string());
                    }
                }
                if value.get("type").and_then(Value::as_str) == Some("error") {
                    let message = value
                        .get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(Value::as_str)
                        .unwrap_or("供应商流式错误");
                    return Err(AppError::new("provider_unavailable", message));
                }
            }
        }
    }

    Ok(LlmOutcome {
        text,
        reasoning,
        input_tokens,
        output_tokens,
        cancelled: false,
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Gemini
// ---------------------------------------------------------------------------

fn gemini_url(base: &str, model: &str, key: &str) -> String {
    let base = base.trim_end_matches('/');
    let base = if base.ends_with("/v1beta") {
        base.to_string()
    } else {
        format!("{base}/v1beta")
    };
    format!(
        "{base}/models/{model}:streamGenerateContent?alt=sse&key={key}"
    )
}

async fn gemini_stream(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    cancel: Arc<AtomicBool>,
    sink: &mut StreamSink<'_>,
) -> AppResult<LlmOutcome> {
    let key = api_key.ok_or_else(|| AppError::new("key_missing", "这个模型还没有设置 API Key"))?;
    let system = messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let contents = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            json!({
                "role": if m.role == "assistant" { "model" } else { "user" },
                "parts": [{ "text": m.content }]
            })
        })
        .collect::<Vec<_>>();
    let body = json!({
        "systemInstruction": { "parts": [{ "text": system }] },
        "contents": contents,
        "generationConfig": { "maxOutputTokens": model.max_output }
    });
    let response = client
        .post(gemini_url(&model.base_url, &model.model, key))
        .json(&body)
        .send()
        .await
        .map_err(request_error)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(provider_error(status, &body));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut text = String::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Ok(LlmOutcome {
                text,
                input_tokens,
                output_tokens,
                cancelled: true,
                ..Default::default()
            });
        }
        let chunk = chunk.map_err(request_error)?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line.trim_start_matches("data:").trim();
            if data.is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(data) {
                if let Some(usage) = value.get("usageMetadata") {
                    input_tokens = usage
                        .get("promptTokenCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(input_tokens);
                    output_tokens = usage
                        .get("candidatesTokenCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(output_tokens);
                }
                if let Some(parts) = value
                    .get("candidates")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(Value::as_array)
                {
                    for part in parts {
                        if let Some(content) = part.get("text").and_then(Value::as_str) {
                            text.push_str(content);
                            (sink.on_delta)(content.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(LlmOutcome {
        text,
        input_tokens,
        output_tokens,
        cancelled: false,
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

pub async fn test_model(
    client: &reqwest::Client,
    _profile_id: &str,
    model: &ModelProfileData,
    api_key: Option<&str>,
) -> AppResult<ModelTestResult> {
    let started = Instant::now();
    if model.provider == "mock" {
        return Ok(ModelTestResult {
            ok: true,
            latency_ms: 0,
            message: "Mock model is available offline".into(),
            model: model.model.clone(),
            model_count: Some(1),
        });
    }
    let key = api_key.ok_or_else(|| AppError::new("key_missing", "这个模型还没有设置 API Key"))?;
    match model.provider.as_str() {
        "openai_compatible" => {
            let url = openai_models_url(&model.base_url);
            let response = client.get(&url).bearer_auth(key).send().await.map_err(request_error)?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(provider_error(status, &body));
            }
            let value = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
            let count = value
                .get("data")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            Ok(ModelTestResult {
                ok: true,
                latency_ms: started.elapsed().as_millis() as u64,
                message: "Connected".into(),
                model: model.model.clone(),
                model_count: Some(count),
            })
        }
        "anthropic" => {
            let body = json!({
                "model": model.model,
                "max_tokens": 1,
                "messages": [{ "role": "user", "content": "ping" }]
            });
            let response = client
                .post(anthropic_url(&model.base_url))
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(request_error)?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(provider_error(status, &body));
            }
            Ok(ModelTestResult {
                ok: true,
                latency_ms: started.elapsed().as_millis() as u64,
                message: "Connected".into(),
                model: model.model.clone(),
                model_count: None,
            })
        }
        "gemini" => {
            let url = format!(
                "{}/v1beta/models?key={key}",
                model.base_url.trim_end_matches('/').trim_end_matches("/v1beta")
            );
            let response = client.get(&url).send().await.map_err(request_error)?;
            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(provider_error(status, &body));
            }
            let value = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
            let count = value
                .get("models")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            Ok(ModelTestResult {
                ok: true,
                latency_ms: started.elapsed().as_millis() as u64,
                message: "Connected".into(),
                model: model.model.clone(),
                model_count: Some(count),
            })
        }
        _ => Err(AppError::unsupported(format!("未知供应商：{}", model.provider))),
    }
}

pub async fn list_models(
    client: &reqwest::Client,
    model: &ModelProfileData,
    api_key: Option<&str>,
) -> AppResult<Vec<String>> {
    if model.provider != "openai_compatible" {
        return Ok(vec![model.model.clone()]);
    }
    let key = api_key.ok_or_else(|| AppError::new("key_missing", "这个模型还没有设置 API Key"))?;
    let url = openai_models_url(&model.base_url);
    let response = client.get(&url).bearer_auth(key).send().await.map_err(request_error)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(provider_error(status, &body));
    }
    let value = response.json::<Value>().await.unwrap_or_else(|_| json!({}));
    Ok(value
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}
