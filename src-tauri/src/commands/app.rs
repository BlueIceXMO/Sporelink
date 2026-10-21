use crate::domain::dto::*;
use crate::error::AppResult;
use crate::state::{AppState, APP_VERSION};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Bootstrap {
    state.bootstrap()
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> AppResult<SettingsState> {
    Ok(state.inner().inner()?.settings_state())
}

#[tauri::command]
pub fn settings_update(
    state: State<'_, AppState>,
    settings: Settings,
    expected_revision: u64,
) -> AppResult<SettingsState> {
    state.inner().inner()?.update_settings(settings, expected_revision)
}

#[tauri::command]
pub fn diagnostics(state: State<'_, AppState>) -> AppResult<Diagnostics> {
    let inner = state.inner().inner()?;
    let models = inner.config.read().model_profiles.len();
    Ok(Diagnostics {
        version: APP_VERSION.into(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        data_dir: inner.workspace.root.to_string_lossy().to_string(),
        db_path: inner.store.db_path.to_string_lossy().to_string(),
        counts: inner.store.counts(),
        keychain_available: inner.keychain.available(),
        model_profiles: models,
        recent_error: None,
    })
}

#[tauri::command]
pub fn workspace_info(state: State<'_, AppState>) -> AppResult<WorkspaceInfo> {
    Ok(state.inner().inner()?.workspace.info())
}

#[tauri::command]
pub fn workspace_open(state: State<'_, AppState>, path: String) -> AppResult<Bootstrap> {
    state.switch_workspace(PathBuf::from(path))
}

#[tauri::command]
pub fn workspace_default(state: State<'_, AppState>) -> AppResult<Bootstrap> {
    state.open_default_workspace()
}

#[tauri::command]
pub fn workspace_file_list(state: State<'_, AppState>) -> AppResult<Vec<WorkspaceFile>> {
    let inner = state.inner().inner()?;
    let config = inner.config.read();
    inner.workspace.list_user_files(&config.ignore_patterns)
}

#[tauri::command]
pub fn workspace_read_file(
    state: State<'_, AppState>,
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> AppResult<String> {
    let inner = state.inner().inner()?;
    inner.workspace.read_text(&path, start_line, end_line)
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, AppState>) -> AppResult<()> {
    let inner = state.inner().inner()?;
    let path = inner.workspace.root.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer").arg(&path).spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(&path).spawn();
    result.map_err(|e| crate::error::AppError::io(e.to_string()))?;
    Ok(())
}
