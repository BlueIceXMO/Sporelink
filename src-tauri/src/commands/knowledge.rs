use crate::context::ChatMessage;
use crate::domain::dto::*;
use crate::error::AppResult;
use crate::service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn knowledge_list(
    state: State<'_, AppState>,
    project_id: Option<String>,
    query: Option<String>,
    tag: Option<String>,
) -> AppResult<Vec<KnowledgeDto>> {
    Ok(service::knowledge_list(
        &state.inner().inner()?.store,
        project_id.as_deref(),
        query.as_deref(),
        tag.as_deref(),
    ))
}

#[tauri::command]
pub fn knowledge_get(state: State<'_, AppState>, id: String) -> AppResult<KnowledgeDto> {
    service::knowledge_get(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn knowledge_create(
    state: State<'_, AppState>,
    input: KnowledgeCreateInput,
) -> AppResult<KnowledgeDto> {
    service::knowledge_create(&state.inner().inner()?.store, input)
}

#[tauri::command]
pub fn knowledge_update(
    state: State<'_, AppState>,
    id: String,
    patch: KnowledgePatch,
    expected_revision: u64,
) -> AppResult<KnowledgeDto> {
    service::knowledge_update(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn knowledge_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::knowledge_delete(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn knowledge_from_message(
    state: State<'_, AppState>,
    message_id: String,
    selection: String,
    title: Option<String>,
) -> AppResult<KnowledgeDto> {
    service::knowledge_from_message(
        &state.inner().inner()?.store,
        KnowledgeFromMessageInput {
            message_id,
            selection,
            title,
        },
    )
}

#[tauri::command]
pub async fn knowledge_check(
    state: State<'_, AppState>,
    input: KnowledgeCheckInput,
) -> AppResult<KnowledgeCheckResult> {
    let inner = state.inner().inner()?;
    let knowledge = service::knowledge_get(&inner.store, &input.id)?;
    let model_id = input.model_profile_id.or_else(|| {
        inner
            .settings_state()
            .settings
            .model_assignments
            .chat
            .clone()
    });
    let profile = match model_id {
        Some(id) => inner.model_profile(&id)?,
        None => {
            let config = inner.config.read();
            config
                .model_profiles
                .iter()
                .find(|profile| profile.data.provider == "mock")
                .or_else(|| config.model_profiles.first())
                .cloned()
                .ok_or_else(|| crate::error::AppError::new("model_not_configured", "没有可用模型"))?
        }
    };
    let model = profile.data.clone();
    let key = inner.api_key(&profile.id)?;
    let prompt = format!(
        "请检查下面这条知识条目是否清晰、准确、可引用。指出偏差、遗漏与一个可验证的下一步。\n\n标题：{}\n\n正文：\n{}",
        knowledge.title, knowledge.body
    );
    let outcome = crate::llm::complete(
        &inner.http,
        &model,
        key.as_deref(),
        &[ChatMessage {
            role: "user".into(),
            content: prompt,
        }],
        "knowledge_check",
    )
    .await?;
    Ok(KnowledgeCheckResult {
        feedback: outcome.text,
        model_profile_id: profile.id,
    })
}
