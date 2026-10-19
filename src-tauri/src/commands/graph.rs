use crate::context;
use crate::domain::dto::*;
use crate::error::{AppError, AppResult};
use crate::service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn graph_get(state: State<'_, AppState>, project_id: String) -> AppResult<GraphView> {
    service::graph_view(&state.inner().inner()?.store, &project_id)
}

#[tauri::command]
pub fn card_get(state: State<'_, AppState>, id: String) -> AppResult<CardDetail> {
    service::card_detail(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn card_create(state: State<'_, AppState>, input: CardCreateInput) -> AppResult<CardDetail> {
    service::create_card(&state.inner().inner()?.store, input)
}

#[tauri::command]
pub fn card_update(
    state: State<'_, AppState>,
    id: String,
    patch: CardPatch,
    expected_revision: u64,
) -> AppResult<CardDetail> {
    service::update_card(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn card_delete(state: State<'_, AppState>, id: String, mode: String) -> AppResult<()> {
    service::delete_card(&state.inner().inner()?.store, &id, &mode)
}

#[tauri::command]
pub fn card_restore(state: State<'_, AppState>, id: String) -> AppResult<CardDetail> {
    service::restore_card(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn card_move(
    state: State<'_, AppState>,
    id: String,
    position: [f64; 3],
) -> AppResult<CardDetail> {
    service::move_card(&state.inner().inner()?.store, &id, position)
}

#[tauri::command]
pub fn card_favorite(state: State<'_, AppState>, id: String, value: bool) -> AppResult<CardDetail> {
    service::set_card_flag(&state.inner().inner()?.store, &id, "favorite", value)
}

#[tauri::command]
pub fn card_lock(state: State<'_, AppState>, id: String, value: bool) -> AppResult<CardDetail> {
    service::set_card_flag(&state.inner().inner()?.store, &id, "locked", value)
}

#[tauri::command]
pub fn turn_update(
    state: State<'_, AppState>,
    id: String,
    patch: TurnPatch,
    expected_revision: u64,
) -> AppResult<TurnDto> {
    service::update_turn(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn turn_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete_turn(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn turn_restore(state: State<'_, AppState>, id: String) -> AppResult<TurnDto> {
    service::restore_turn(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn message_update(
    state: State<'_, AppState>,
    id: String,
    patch: MessagePatch,
    expected_revision: u64,
) -> AppResult<MessageDto> {
    service::update_message(&state.inner().inner()?.store, &id, patch, expected_revision)
}

#[tauri::command]
pub fn message_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete_message(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn message_restore(state: State<'_, AppState>, id: String) -> AppResult<MessageDto> {
    service::restore_message(&state.inner().inner()?.store, &id)
}

#[tauri::command]
pub fn message_favorite(
    state: State<'_, AppState>,
    id: String,
    value: bool,
) -> AppResult<MessageDto> {
    service::set_message_favorite(&state.inner().inner()?.store, &id, value)
}

#[tauri::command]
pub fn context_preview(
    state: State<'_, AppState>,
    input: ContextPreviewInput,
) -> AppResult<ContextPreview> {
    let inner = state.inner().inner()?;
    service::context_preview(&inner, input)
}

#[tauri::command]
pub fn context_snapshot_get(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<ContextSnapshotDto> {
    let inner = state.inner().inner()?;
    let node = inner
        .store
        .live_node(&id)
        .filter(|node| node.node_type == crate::domain::model::NodeType::ContextSnapshot)
        .ok_or_else(|| AppError::not_found("context snapshot"))?;
    let items = serde_json::from_value(
        node.data
            .get("items")
            .cloned()
            .unwrap_or(serde_json::json!([])),
    )
    .unwrap_or_default();
    Ok(ContextSnapshotDto {
        id: node.id.clone(),
        card_id: node.data_str("cardId").unwrap_or("").into(),
        turn_id: node.data_str("turnId").map(str::to_string),
        model_profile_id: node.data_str("modelProfileId").unwrap_or("").into(),
        items,
        total_tokens: node.data.get("totalTokens").and_then(|v| v.as_u64()).unwrap_or(0),
        created_at: node.created_at,
    })
}

// Keep the unused import warning-free; the context module is used by chat.
#[allow(dead_code)]
fn _context_type_hint(_: &context::CompiledContext) {}
