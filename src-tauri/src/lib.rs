pub mod commands;
pub mod context;
pub mod documents;
pub mod domain;
pub mod error;
pub mod export;
pub mod keychain;
pub mod llm;
pub mod sample;
pub mod search;
pub mod service;
pub mod state;
pub mod store;

use commands::*;
use state::AppState;
use tauri::Manager;

pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sporelink=info".into()),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = AppState::new(app.handle());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            settings_get,
            settings_update,
            diagnostics,
            open_data_dir,
            project_list,
            project_get,
            project_create,
            project_update,
            project_delete,
            project_restore,
            project_duplicate,
            project_summary,
            project_export,
            project_import,
            folder_list,
            folder_create,
            folder_update,
            folder_delete,
            favorites_export,
            graph_get,
            card_get,
            card_create,
            card_update,
            card_delete,
            card_restore,
            card_move,
            card_favorite,
            card_lock,
            turn_update,
            turn_delete,
            turn_restore,
            message_update,
            message_delete,
            message_restore,
            message_favorite,
            context_preview,
            context_snapshot_get,
            chat_send,
            chat_regenerate,
            chat_cancel,
            search,
            document_import,
            document_import_url,
            document_list,
            document_get,
            document_delete,
            document_search,
            knowledge_list,
            knowledge_get,
            knowledge_create,
            knowledge_update,
            knowledge_delete,
            knowledge_from_message,
            knowledge_check,
            model_profile_list,
            model_profile_create,
            model_profile_update,
            model_profile_delete,
            model_test,
            model_list_remote,
            secret_set,
            secret_status,
            secret_delete,
            vault_export,
            vault_import,
            vault_clear,
            snapshot_create,
            snapshot_list,
            snapshot_restore,
            trash_list,
            trash_restore,
            sample_install,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Ok(inner) = state.inner().inner() {
                        let _ = inner.store.flush();
                    }
                }
            }
        });
}
