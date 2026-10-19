use crate::domain::dto::*;
use crate::domain::model::{now_ms, Node, NodeType};
use crate::error::{AppError, AppResult};
use crate::llm;
use crate::service;
use crate::state::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn model_profile_list(state: State<'_, AppState>) -> AppResult<Vec<ModelProfileDto>> {
    service::list_model_profiles(&state.inner().inner()?.store)
}

#[tauri::command]
pub fn model_profile_create(
    state: State<'_, AppState>,
    input: ModelProfileData,
) -> AppResult<ModelProfileDto> {
    let inner = state.inner().inner()?;
    if input.label.trim().is_empty() || input.model.trim().is_empty() {
        return Err(AppError::invalid("模型名称与模型 ID 不能为空"));
    }
    let node = Node::new(
        NodeType::ModelProfile,
        None,
        input.label.clone(),
        "",
        serde_json::to_value(&input)?,
    );
    let node = inner.store.put_node(node, None)?;
    Ok(ModelProfileDto {
        id: node.id,
        revision: node.revision,
        data: input,
    })
}

#[tauri::command]
pub fn model_profile_update(
    state: State<'_, AppState>,
    id: String,
    patch: ModelProfilePatch,
    expected_revision: u64,
) -> AppResult<ModelProfileDto> {
    let inner = state.inner().inner()?;
    let mut node = inner
        .store
        .live_node(&id)
        .filter(|node| node.node_type == NodeType::ModelProfile)
        .ok_or_else(|| AppError::not_found("model profile"))?;
    let mut data = serde_json::from_value::<ModelProfileData>(node.data.clone())
        .map_err(|e| AppError::parse(e.to_string()))?;
    if let Some(label) = patch.label {
        data.label = label.clone();
        node.title = label;
    }
    if let Some(base_url) = patch.base_url {
        data.base_url = base_url;
    }
    if let Some(model) = patch.model {
        data.model = model;
    }
    if let Some(context_window) = patch.context_window {
        data.context_window = context_window;
    }
    if let Some(max_output) = patch.max_output {
        data.max_output = max_output;
    }
    if let Some(capabilities) = patch.capabilities {
        data.capabilities = capabilities;
    }
    if let Some(pricing) = patch.pricing {
        data.pricing = pricing;
    }
    if let Some(roles) = patch.roles {
        data.roles = roles;
    }
    node.data = serde_json::to_value(&data)?;
    let node = inner.store.put_node(node, Some(expected_revision))?;
    Ok(ModelProfileDto {
        id: node.id,
        revision: node.revision,
        data,
    })
}

#[tauri::command]
pub fn model_profile_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let inner = state.inner().inner()?;
    let mut node = inner
        .store
        .live_node(&id)
        .filter(|node| node.node_type == NodeType::ModelProfile)
        .ok_or_else(|| AppError::not_found("model profile"))?;
    if node.data_bool("isBuiltin").unwrap_or(false) {
        return Err(AppError::new("invalid_input", "内置 Mock 模型不能删除"));
    }
    node.deleted_at = Some(now_ms());
    inner.store.put_node(node, None)?;
    let _ = inner.keychain.delete(&id);
    Ok(())
}

#[tauri::command]
pub async fn model_test(state: State<'_, AppState>, id: String) -> AppResult<ModelTestResult> {
    let inner = state.inner().inner()?;
    let (node, model) = inner.model_profile(&id)?;
    let key = inner.api_key(&node.id)?;
    llm::test_model(&inner.http, &id, &model, key.as_deref()).await
}

#[tauri::command]
pub async fn model_list_remote(state: State<'_, AppState>, id: String) -> AppResult<Vec<String>> {
    let inner = state.inner().inner()?;
    let (node, model) = inner.model_profile(&id)?;
    let key = inner.api_key(&node.id)?;
    llm::list_models(&inner.http, &model, key.as_deref()).await
}

#[tauri::command]
pub fn secret_set(
    state: State<'_, AppState>,
    profile_id: String,
    key: String,
) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    let _ = inner.model_profile(&profile_id)?;
    inner.keychain.set(&profile_id, &key)?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub fn secret_status(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    let configured = inner.keychain.status(&profile_id);
    Ok(json!({ "configured": configured }))
}

#[tauri::command]
pub fn secret_delete(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    inner.keychain.delete(&profile_id)?;
    Ok(json!({ "ok": true }))
}
