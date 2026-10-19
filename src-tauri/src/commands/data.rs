use crate::domain::dto::*;
use crate::error::AppResult;
use crate::export;
use crate::sample;
use crate::service;
use crate::state::AppState;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn search(state: State<'_, AppState>, input: SearchInput) -> AppResult<SearchResults> {
    Ok(crate::search::search(&state.inner().inner()?.store, &input))
}

#[tauri::command]
pub fn document_import(
    state: State<'_, AppState>,
    path: String,
    project_id: Option<String>,
) -> AppResult<DocumentDetail> {
    let inner = state.inner().inner()?;
    crate::documents::import_document(&inner.store, Path::new(&path), project_id)
}

#[tauri::command]
pub async fn document_import_url(
    state: State<'_, AppState>,
    url: String,
    project_id: Option<String>,
) -> AppResult<DocumentDetail> {
    let inner = state.inner().inner()?;
    crate::documents::import_url(&inner.http, &inner.store, &url, project_id).await
}

#[tauri::command]
pub fn document_list(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> AppResult<Vec<DocumentSummary>> {
    Ok(crate::documents::list_documents(
        &state.inner().inner()?.store,
        project_id.as_deref(),
    ))
}

#[tauri::command]
pub fn document_get(state: State<'_, AppState>, id: String) -> AppResult<DocumentDetail> {
    crate::documents::document_detail(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn document_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    crate::documents::delete_document(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn document_search(
    state: State<'_, AppState>,
    id: String,
    query: String,
) -> AppResult<Vec<ChunkMatch>> {
    Ok(crate::documents::search_document(&state.inner().inner()?.store, &id, &query))
}

#[tauri::command]
pub fn vault_export(state: State<'_, AppState>, path: String) -> AppResult<serde_json::Value> {
    let inner = state.inner().inner()?;
    let payload = export::vault_payload(&inner.store);
    export::write_json(Path::new(&path), &payload)?;
    Ok(serde_json::json!({ "path": path }))
}

#[tauri::command]
pub fn vault_import(
    state: State<'_, AppState>,
    path: String,
    mode: String,
) -> AppResult<ImportResult> {
    let inner = state.inner().inner()?;
    let value = export::read_json(Path::new(&path))?;
    if mode == "replace" {
        inner.store.clear_all()?;
        let result = service::import_graph(&inner.store, &value, None)?;
        let _ = inner.reload_settings();
        Ok(result)
    } else {
        service::import_graph(&inner.store, &value, None)
    }
}

#[tauri::command]
pub fn vault_clear(state: State<'_, AppState>, confirm: String) -> AppResult<()> {
    if confirm != "DELETE" {
        return Err(crate::error::AppError::invalid("请输入 DELETE 以确认清除"));
    }
    let inner = state.inner().inner()?;
    inner.store.clear_all()?;
    let _ = inner.reload_settings();
    Ok(())
}

#[tauri::command]
pub fn snapshot_create(
    state: State<'_, AppState>,
    label: Option<String>,
) -> AppResult<SnapshotSummary> {
    let inner = state.inner().inner()?;
    service::snapshot_create(&inner.data_dir, &inner.store, label)
}

#[tauri::command]
pub fn snapshot_list(state: State<'_, AppState>) -> AppResult<Vec<SnapshotSummary>> {
    let inner = state.inner().inner()?;
    Ok(service::snapshot_list(&inner.data_dir))
}

#[tauri::command]
pub fn snapshot_restore(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let inner = state.inner().inner()?;
    service::snapshot_restore(&inner.data_dir, &inner.store, &id)?;
    let _ = inner.reload_settings();
    Ok(())
}

#[tauri::command]
pub fn trash_list(state: State<'_, AppState>) -> AppResult<Vec<TrashItem>> {
    Ok(service::trash_list(&state.inner().inner()?.store))
}

#[tauri::command]
pub fn trash_restore(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::trash_restore(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn sample_install(state: State<'_, AppState>, locale: String) -> AppResult<ProjectDetail> {
    let inner = state.inner().inner()?;
    let project_id = sample::install_sample(&inner.store, &locale)?;
    service::project_detail(&inner.store, &project_id)
}
