use crate::domain::dto::*;
use crate::error::AppResult;
use crate::export;
use crate::service;
use crate::state::AppState;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn project_list(state: State<'_, AppState>, include_archived: bool) -> AppResult<Vec<ProjectSummary>> {
    Ok(service::list_projects(&state.inner().inner()?.store, include_archived))
}

#[tauri::command]
pub fn project_get(state: State<'_, AppState>, id: String) -> AppResult<ProjectDetail> {
    service::project_detail(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn project_create(state: State<'_, AppState>, input: ProjectCreateInput) -> AppResult<ProjectDetail> {
    service::create_project(&state.inner().inner()?.store, input)
}

#[tauri::command]
pub fn project_update(
    state: State<'_, AppState>,
    id: String,
    patch: ProjectPatch,
    expected_revision: u64,
) -> AppResult<ProjectDetail> {
    service::update_project(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn project_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete_project(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn project_restore(state: State<'_, AppState>, id: String) -> AppResult<ProjectDetail> {
    service::restore_project(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn project_duplicate(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
) -> AppResult<ProjectDetail> {
    service::duplicate_project(&state.inner().inner()?.store, &id, name)
}

#[tauri::command]
pub async fn project_summary(
    state: State<'_, AppState>,
    id: String,
    model_profile_id: Option<String>,
) -> AppResult<KnowledgeDto> {
    let inner = state.inner().inner()?;
    service::project_summary_generate(&inner, &id, model_profile_id).await
}

#[tauri::command]
pub fn project_export(
    state: State<'_, AppState>,
    id: String,
    path: String,
    format: String,
) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    let path = Path::new(&path);
    if format == "markdown" || format == "md" {
        let markdown = export::project_markdown(&inner.store, &id)?;
        std::fs::write(path, markdown)?;
    } else {
        let payload = export::project_payload(&inner.store, &id)?;
        export::write_json(path, &payload)?;
    }
    Ok(serde_json::json!({ "path": path.to_string_lossy() }))
}

#[tauri::command]
pub fn project_import(
    state: State<'_, AppState>,
    path: String,
    folder_id: Option<String>,
) -> AppResult<ProjectDetail> {
    service::project_import(&state.inner().inner()?.store, Path::new(&path), folder_id)
}

#[tauri::command]
pub fn folder_list(state: State<'_, AppState>) -> AppResult<Vec<FolderSummary>> {
    Ok(service::list_folders(&state.inner().inner()?.store))
}

#[tauri::command]
pub fn folder_create(state: State<'_, AppState>, name: String, parent_id: Option<String>) -> AppResult<FolderSummary> {
    service::create_folder(&state.inner().inner()?.store, FolderCreateInput { name, parent_id })
}

#[tauri::command]
pub fn folder_update(
    state: State<'_, AppState>,
    id: String,
    patch: FolderPatch,
    expected_revision: u64,
) -> AppResult<FolderSummary> {
    service::update_folder(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn folder_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete_folder(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn favorites_export(
    state: State<'_, AppState>,
    path: String,
    format: String,
) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    let results = crate::search::search(
        &inner.store,
        &SearchInput {
            query: String::new(),
            project_id: None,
            types: None,
            favorite_only: Some(true),
            limit: Some(200),
        },
    );
    if format == "markdown" || format == "md" {
        let mut out = String::from("# Sporelink Favorites\n\n");
        for hit in results.hits {
            out.push_str(&format!("## {}\n\n{}\n\n", hit.title, hit.snippet));
        }
        std::fs::write(&path, out)?;
    } else {
        let payload = serde_json::json!({
            "format": "sporelink.favorites",
            "schemaVersion": 1,
            "exportedAt": crate::domain::model::now_ms(),
            "hits": results.hits,
        });
        export::write_json(Path::new(&path), &payload)?;
    }
    Ok(serde_json::json!({ "path": path }))
}
