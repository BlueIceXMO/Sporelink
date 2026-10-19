use crate::domain::dto::{
    Bootstrap, Counts, ModelCapabilities, ModelPricing, ModelProfileData, Settings, SettingsState,
    VaultInfo,
};
use crate::domain::model::{new_id, Node, NodeType};
use crate::error::{AppError, AppResult};
use crate::keychain::Keychain;
use crate::llm;
use crate::service;
use crate::store::Store;
use parking_lot::RwLock;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;

pub const APP_VERSION: &str = "0.1.0-alpha";

#[derive(Clone)]
pub struct SettingsCache {
    pub settings: Settings,
    pub node_id: String,
    pub revision: u64,
}

pub struct AppInner {
    pub store: Arc<Store>,
    pub settings: RwLock<SettingsCache>,
    pub vault: VaultInfo,
    pub data_dir: PathBuf,
    pub keychain: Keychain,
    pub http: reqwest::Client,
    pub cancel: parking_lot::Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AppInner {
    pub fn settings_state(&self) -> SettingsState {
        let cache = self.settings.read();
        SettingsState {
            settings: cache.settings.clone(),
            revision: cache.revision,
        }
    }

    pub fn update_settings(&self, settings: Settings, expected_revision: u64) -> AppResult<SettingsState> {
        let cache = self.settings.read();
        if cache.revision != expected_revision {
            return Err(AppError::revision_conflict(expected_revision, cache.revision));
        }
        let node_id = cache.node_id.clone();
        drop(cache);

        let mut node = self
            .store
            .live_node(&node_id)
            .ok_or_else(|| AppError::not_found("settings"))?;
        node.data = serde_json::to_value(&settings)?;
        let node = self.store.put_node(node, Some(expected_revision))?;
        let mut cache = self.settings.write();
        cache.settings = settings;
        cache.revision = node.revision;
        Ok(SettingsState {
            settings: cache.settings.clone(),
            revision: cache.revision,
        })
    }

    pub fn reload_settings(&self) -> AppResult<SettingsState> {
        let (cache, _) = service::ensure_seed(&self.store, &self.keychain)?;
        let state = SettingsState {
            settings: cache.settings.clone(),
            revision: cache.revision,
        };
        *self.settings.write() = cache;
        Ok(state)
    }

    pub fn model_profile(&self, id: &str) -> AppResult<(Node, ModelProfileData)> {
        let node = self
            .store
            .live_node(id)
            .filter(|node| node.node_type == NodeType::ModelProfile)
            .ok_or_else(|| AppError::not_found("model profile"))?;
        let data = serde_json::from_value::<ModelProfileData>(node.data.clone())
            .map_err(|e| AppError::parse(format!("模型配置损坏：{e}")))?;
        Ok((node, data))
    }

    pub fn api_key(&self, profile_id: &str) -> AppResult<Option<String>> {
        self.keychain.get(profile_id)
    }
}

impl AppInner {
    pub fn cancel_token(&self, run_id: &str) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        self.cancel.lock().insert(run_id.to_string(), token.clone());
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
    pub inner: Result<Arc<AppInner>, AppError>,
    pub fallback_data_dir: PathBuf,
}

impl AppState {
    pub fn new(app: &tauri::AppHandle) -> Self {
        let data_dir = std::env::var("SPORELINK_DATA_DIR")
            .map(PathBuf::from)
            .ok()
            .or_else(|| app.path().app_data_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let fallback_data_dir = data_dir.clone();
        match init_inner(app, &data_dir) {
            Ok(inner) => Self {
                inner: Ok(Arc::new(inner)),
                fallback_data_dir,
            },
            Err(error) => {
                tracing::error!("failed to initialize app: {error}");
                Self {
                    inner: Err(error),
                    fallback_data_dir,
                }
            }
        }
    }

    pub fn inner(&self) -> AppResult<Arc<AppInner>> {
        self.inner.clone()
    }

    pub fn fallback_bootstrap(&self) -> Bootstrap {
        let vault = VaultInfo {
            id: "default".into(),
            name: "Sporelink".into(),
            path: self.fallback_data_dir.to_string_lossy().to_string(),
        };
        let error = self.inner.as_ref().err().cloned();
        Bootstrap {
            version: APP_VERSION.into(),
            vault,
            settings: Settings::default(),
            settings_revision: 0,
            projects: vec![],
            folders: vec![],
            models: vec![],
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
            init_error: error,
        }
    }
}

fn init_inner(app: &tauri::AppHandle, data_dir: &PathBuf) -> AppResult<AppInner> {
    std::fs::create_dir_all(data_dir).map_err(|e| {
        AppError::io(format!("创建数据目录失败 {}: {e}", data_dir.display()))
    })?;
    let vault_dir = data_dir.join("vaults").join("default");
    std::fs::create_dir_all(&vault_dir).map_err(|e| {
        AppError::io(format!("创建 Vault 目录失败 {}: {e}", vault_dir.display()))
    })?;
    let meta_path = data_dir.join("app.json");
    let vault_id = if meta_path.exists() {
        let text = std::fs::read_to_string(&meta_path).unwrap_or_default();
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.get("vaultId").and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or_else(new_id)
    } else {
        new_id()
    };
    let meta = json!({ "vaultId": vault_id, "schemaVersion": 1 });
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?).map_err(|e| {
        AppError::io(format!("写入 app.json 失败 {}: {e}", meta_path.display()))
    })?;

    let store = Arc::new(Store::open(&vault_dir).map_err(|e| {
        AppError::io(format!("打开 Vault 数据库失败 {}: {e}", vault_dir.display()))
    })?);
    let keychain = Keychain::new();
    let (settings_cache, _) = service::ensure_seed(&store, &keychain)?;
    let inner = AppInner {
        store,
        settings: RwLock::new(settings_cache),
        vault: VaultInfo {
            id: vault_id,
            name: "Sporelink Vault".into(),
            path: vault_dir.to_string_lossy().to_string(),
        },
        data_dir: data_dir.clone(),
        keychain,
        http: llm::http_client(),
        cancel: parking_lot::Mutex::new(HashMap::new()),
    };
    let _ = app;
    Ok(inner)
}

pub fn mock_profile() -> ModelProfileData {
    ModelProfileData {
        provider: "mock".into(),
        label: "Mock (offline)".into(),
        base_url: String::new(),
        model: "mock-1".into(),
        context_window: 128_000,
        max_output: 8192,
        capabilities: ModelCapabilities {
            vision: false,
            tools: false,
            json: true,
            reasoning: true,
        },
        pricing: ModelPricing {
            input_per_million: 0.0,
            output_per_million: 0.0,
            currency: "USD".into(),
        },
        roles: vec!["chat".into(), "reasoning".into(), "title".into(), "summary".into()],
        is_builtin: true,
    }
}

pub fn settings_node(settings: &Settings) -> Node {
    Node::new(
        NodeType::Settings,
        None,
        "Settings",
        "",
        serde_json::to_value(settings).unwrap_or(json!({})),
    )
}
