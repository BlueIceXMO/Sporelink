use crate::domain::dto::*;
use crate::domain::model::new_id;
use crate::error::{AppError, AppResult};
use crate::llm;
use crate::state::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn model_profile_list(state: State<'_, AppState>) -> AppResult<Vec<ModelProfileDto>> {
    Ok(state.app_config().model_profiles)
}

#[tauri::command]
pub fn model_profile_create(
    state: State<'_, AppState>,
    input: ModelProfileData,
) -> AppResult<ModelProfileDto> {
    if input.label.trim().is_empty() || input.model.trim().is_empty() {
        return Err(AppError::invalid("模型名称与模型 ID 不能为空"));
    }
    let profile = ModelProfileDto {
        id: new_id(),
        revision: 1,
        data: input,
    };
    let created = profile.clone();
    state.update_config(|config| config.model_profiles.push(profile))?;
    Ok(created)
}

#[tauri::command]
pub fn model_profile_update(
    state: State<'_, AppState>,
    id: String,
    patch: ModelProfilePatch,
    expected_revision: u64,
) -> AppResult<ModelProfileDto> {
    let current = state.app_config();
    let mut profile = current.model_profile(&id)?;
    if profile.revision != expected_revision {
        return Err(AppError::revision_conflict(
            expected_revision,
            profile.revision,
        ));
    }
    if let Some(label) = patch.label {
        profile.data.label = label;
    }
    if let Some(base_url) = patch.base_url {
        profile.data.base_url = base_url;
    }
    if let Some(model) = patch.model {
        profile.data.model = model;
    }
    if let Some(context_window) = patch.context_window {
        profile.data.context_window = context_window;
    }
    if let Some(max_output) = patch.max_output {
        profile.data.max_output = max_output;
    }
    if let Some(capabilities) = patch.capabilities {
        profile.data.capabilities = capabilities;
    }
    if let Some(pricing) = patch.pricing {
        profile.data.pricing = pricing;
    }
    if let Some(roles) = patch.roles {
        profile.data.roles = roles;
    }
    profile.revision = profile.revision.saturating_add(1);
    let updated = profile.clone();
    state.update_config(|config| config.save_model_profile(profile))?;
    Ok(updated)
}

#[tauri::command]
pub fn model_profile_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let current = state.app_config();
    let profile = current.model_profile(&id)?;
    if profile.data.is_builtin {
        return Err(AppError::new("invalid_input", "内置 Mock 模型不能删除"));
    }
    state.update_config(|config| {
        let _ = config.delete_model_profile(&id);
    })?;
    let inner = state.inner().inner()?;
    let _ = inner.keychain.delete(&id);
    Ok(())
}

#[tauri::command]
pub async fn model_test(state: State<'_, AppState>, id: String) -> AppResult<ModelTestResult> {
    let inner = state.inner().inner()?;
    let profile = inner.model_profile(&id)?;
    let key = inner.api_key(&profile.id)?;
    llm::test_model(&inner.http, &id, &profile.data, key.as_deref()).await
}

#[tauri::command]
pub async fn model_list_remote(state: State<'_, AppState>, id: String) -> AppResult<Vec<String>> {
    let inner = state.inner().inner()?;
    let profile = inner.model_profile(&id)?;
    let key = inner.api_key(&profile.id)?;
    llm::list_models(&inner.http, &profile.data, key.as_deref()).await
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
