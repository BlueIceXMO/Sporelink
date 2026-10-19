use crate::domain::dto::*;
use crate::error::AppResult;
use crate::service;
use crate::state::{AppState, APP_VERSION};
use tauri::State;

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Bootstrap {
    match state.inner().inner() {
        Ok(inner) => {
            let settings_state = inner.settings_state();
            Bootstrap {
                version: APP_VERSION.into(),
                vault: inner.vault.clone(),
                settings: settings_state.settings,
                settings_revision: settings_state.revision,
                projects: service::list_projects(&inner.store, true),
                folders: service::list_folders(&inner.store),
                models: service::list_model_profiles(&inner.store).unwrap_or_default(),
                counts: inner.store.counts(),
                keychain_available: inner.keychain.available(),
                init_error: None,
            }
        }
        Err(error) => {
            let mut bootstrap = state.fallback_bootstrap();
            bootstrap.init_error = Some(error);
            bootstrap
        }
    }
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
    let models = service::list_model_profiles(&inner.store)?.len();
    Ok(Diagnostics {
        version: APP_VERSION.into(),
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        data_dir: inner.data_dir.to_string_lossy().to_string(),
        db_path: inner.store.db_path.to_string_lossy().to_string(),
        counts: inner.store.counts(),
        keychain_available: inner.keychain.available(),
        model_profiles: models,
        recent_error: None,
    })
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, AppState>) -> AppResult<()> {
    let inner = state.inner().inner()?;
    let path = inner.data_dir.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer").arg(&path).spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(&path).spawn();
    result.map_err(|e| crate::error::AppError::io(e.to_string()))?;
    Ok(())
}
