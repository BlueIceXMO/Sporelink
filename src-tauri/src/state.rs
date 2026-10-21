use crate::config::AppConfig;
use crate::domain::dto::{
    Bootstrap, Counts, ModelProfileData, ModelProfileDto, Settings, SettingsState, VaultInfo,
    WorkspaceInfo,
};
use crate::domain::model::now_ms;
use crate::error::{AppError, AppResult};
use crate::keychain::Keychain;
use crate::llm;
use crate::paths;
use crate::service;
use crate::store::Store;
use crate::workspace::{AccessMode, Workspace};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub const APP_VERSION: &str = "0.1.0-alpha";

pub struct AppInner {
    pub store: Arc<Store>,
    pub workspace: Arc<Workspace>,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_dir: PathBuf,
    pub vault: VaultInfo,
    pub keychain: Keychain,
    pub http: reqwest::Client,
    pub cancel: parking_lot::Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AppInner {
    pub fn settings_state(&self) -> SettingsState {
        let config = self.config.read();
        SettingsState {
            settings: config.settings.clone(),
            revision: config.settings_revision,
        }
    }

    pub fn update_settings(
        &self,
        settings: Settings,
        expected_revision: u64,
    ) -> AppResult<SettingsState> {
        let mut config = self.config.write();
        if config.settings_revision != expected_revision {
            return Err(AppError::revision_conflict(
                expected_revision,
                config.settings_revision,
            ));
        }
        config.settings = settings;
        config.settings_revision = config.settings_revision.saturating_add(1);
        let revision = config.settings_revision;
        let settings = config.settings.clone();
        config.save(&self.config_dir)?;
        Ok(SettingsState { settings, revision })
    }

    pub fn reload_settings(&self) -> AppResult<SettingsState> {
        let config = AppConfig::load(&self.config_dir)?;
        let state = SettingsState {
            settings: config.settings.clone(),
            revision: config.settings_revision,
        };
        *self.config.write() = config;
        Ok(state)
    }

    pub fn model_profile(&self, id: &str) -> AppResult<ModelProfileDto> {
        self.config.read().model_profile(id)
    }

    pub fn model_data(&self, id: &str) -> AppResult<ModelProfileData> {
        Ok(self.model_profile(id)?.data)
    }

    pub fn api_key(&self, profile_id: &str) -> AppResult<Option<String>> {
        self.keychain.get(profile_id)
    }
}

impl AppInner {
    pub fn cancel_token(&self, run_id: &str) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        self.cancel
            .lock()
            .insert(run_id.to_string(), token.clone());
        token
    }

    pub fn clear_cancel(&self, run_id: &str) {
        self.cancel.lock().remove(run_id);
    }

    pub fn request_cancel(&self, run_id: &str) {
        if let Some(token) = self.cancel.lock().get(run_id) {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

pub struct AppState {
    pub config_dir: PathBuf,
    pub config: Arc<RwLock<AppConfig>>,
    pub current: RwLock<Result<Arc<AppInner>, AppError>>,
    pub fallback_workspace: PathBuf,
}

impl AppState {
    pub fn new(_app: &tauri::AppHandle) -> Self {
        let config_dir = paths::config_dir();
        let _ = std::fs::create_dir_all(&config_dir);
        let mut config = match AppConfig::load(&config_dir) {
            Ok(config) => config,
            Err(error) => {
                paths::append_log(&format!("config load error: {error}"));
                AppConfig::default()
            }
        };
        config.ensure_defaults();

        let env_workspace = std::env::var("SPORELINK_WORKSPACE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        let mut candidates = Vec::new();
        if let Some(path) = env_workspace {
            candidates.push(path);
        } else {
            if let Some(active) = config
                .active_workspace
                .as_ref()
                .map(PathBuf::from)
                .filter(|path| path.is_dir())
            {
                candidates.push(active);
            }
            for candidate in paths::default_workspace_candidates() {
                if !candidates.contains(&candidate) {
                    candidates.push(candidate);
                }
            }
        }
        if candidates.is_empty() {
            candidates.push(paths::default_workspace());
        }

        let config_arc = Arc::new(RwLock::new(config.clone()));
        let access = AccessMode::parse(&config.workspace_access);
        let mut initialized = None;
        let mut last_error = None;
        let mut fallback_workspace = candidates[0].clone();
        for root in &candidates {
            paths::append_log(&format!(
                "startup config_dir={} workspace={} access={}",
                config_dir.display(),
                root.display(),
                access.as_str()
            ));
            match init_inner(config_arc.clone(), config_dir.clone(), root, access) {
                Ok(inner) => {
                    initialized = Some(inner);
                    break;
                }
                Err(error) => {
                    paths::append_log(&format!(
                        "workspace init error root={} error={error}",
                        root.display()
                    ));
                    fallback_workspace = root.clone();
                    last_error = Some(error);
                }
            }
        }
        let current = match initialized {
            Some(inner) => {
                config.touch_workspace(
                    &inner.workspace.root,
                    &inner.workspace.manifest.name,
                    now_ms(),
                );
                let _ = config.save(&config_dir);
                *config_arc.write() = config;
                fallback_workspace = inner.workspace.root.clone();
                Ok(Arc::new(inner))
            }
            None => Err(last_error.unwrap_or_else(|| {
                AppError::io(format!("无法打开默认工作区 {}", fallback_workspace.display()))
            })),
        };

        Self {
            config_dir,
            config: config_arc,
            current: RwLock::new(current),
            fallback_workspace,
        }
    }

    pub fn inner(&self) -> AppResult<Arc<AppInner>> {
        self.current.read().clone()
    }

    pub fn app_config(&self) -> AppConfig {
        self.config.read().clone()
    }

    pub fn update_config<F>(&self, update: F) -> AppResult<AppConfig>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.write();
        update(&mut config);
        config.ensure_defaults();
        config.save(&self.config_dir)?;
        Ok(config.clone())
    }

    pub fn switch_workspace(&self, path: PathBuf) -> AppResult<Bootstrap> {
        if let Ok(current) = self.inner() {
            let _ = current.store.flush();
        }
        let access = AccessMode::parse(&self.config.read().workspace_access);
        let inner = init_inner(
            self.config.clone(),
            self.config_dir.clone(),
            &path,
            access,
        )?;
        {
            let mut config = self.config.write();
            config.touch_workspace(
                &inner.workspace.root,
                &inner.workspace.manifest.name,
                now_ms(),
            );
            config.save(&self.config_dir)?;
        }
        *self.current.write() = Ok(Arc::new(inner));
        Ok(self.bootstrap())
    }

    pub fn open_default_workspace(&self) -> AppResult<Bootstrap> {
        let mut last_error = None;
        for candidate in paths::default_workspace_candidates() {
            match self.switch_workspace(candidate) {
                Ok(bootstrap) => return Ok(bootstrap),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| AppError::io("没有可用的默认工作区")))
    }

    pub fn bootstrap(&self) -> Bootstrap {
        match self.inner() {
            Ok(inner) => {
                let settings_state = inner.settings_state();
                let config = inner.config.read();
                Bootstrap {
                    version: APP_VERSION.into(),
                    vault: inner.vault.clone(),
                    workspace: inner.workspace.info(),
                    settings: settings_state.settings,
                    settings_revision: settings_state.revision,
                    projects: service::list_projects(&inner.store, true),
                    folders: service::list_folders(&inner.store),
                    models: config.model_profiles.clone(),
                    counts: inner.store.counts(),
                    keychain_available: inner.keychain.available(),
                    init_error: None,
                }
            }
            Err(error) => {
                let mut bootstrap = self.fallback_bootstrap();
                bootstrap.init_error = Some(error);
                bootstrap
            }
        }
    }

    pub fn fallback_bootstrap(&self) -> Bootstrap {
        let config = self.config.read();
        let workspace = WorkspaceInfo {
            id: "unavailable".into(),
            name: self
                .fallback_workspace
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "Sporelink".into()),
            root: self.fallback_workspace.to_string_lossy().to_string(),
            internal: self
                .fallback_workspace
                .join(".spore")
                .to_string_lossy()
                .to_string(),
            access: config.workspace_access.clone(),
            created_at: 0,
        };
        let vault = VaultInfo {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            path: workspace.internal.clone(),
        };
        Bootstrap {
            version: APP_VERSION.into(),
            vault,
            workspace,
            settings: config.settings.clone(),
            settings_revision: config.settings_revision,
            projects: vec![],
            folders: vec![],
            models: config.model_profiles.clone(),
            counts: Counts {
                projects: 0,
                cards: 0,
                messages: 0,
                documents: 0,
                knowledge: 0,
                nodes: 0,
                edges: 0,
            },
            keychain_available: false,
            init_error: self.current.read().as_ref().err().cloned(),
        }
    }
}

fn init_inner(
    config: Arc<RwLock<AppConfig>>,
    config_dir: PathBuf,
    root: &std::path::Path,
    access: AccessMode,
) -> AppResult<AppInner> {
    let (workspace, created) = Workspace::open_or_create(root, access)?;
    if created {
        paths::append_log(&format!(
            "created workspace root={} internal={}",
            workspace.root.display(),
            workspace.internal.display()
        ));
    }
    let store = Arc::new(Store::open(&workspace.internal).map_err(|e| {
        AppError::io(format!(
            "打开 Vault 数据库失败 {}: {e}",
            workspace.internal.display()
        ))
    })?);
    let vault = VaultInfo {
        id: workspace.manifest.id.clone(),
        name: workspace.manifest.name.clone(),
        path: workspace.internal.to_string_lossy().to_string(),
    };
    Ok(AppInner {
        store,
        workspace: Arc::new(workspace),
        config,
        config_dir,
        vault,
        keychain: Keychain::new(),
        http: llm::http_client(),
        cancel: parking_lot::Mutex::new(HashMap::new()),
    })
}
