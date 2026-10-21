use crate::domain::dto::{
    ModelCapabilities, ModelPricing, ModelProfileData, ModelProfileDto, Settings,
};
use crate::domain::model::new_id;
use crate::error::{AppError, AppResult};
use crate::paths;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONFIG_SCHEMA_VERSION: u32 = 1;

fn config_schema_version() -> u32 {
    CONFIG_SCHEMA_VERSION
}
pub const DEFAULT_IGNORE: &[&str] = &[
    ".spore",
    ".sporelink",
    ".git",
    "node_modules",
    "target",
    "dist",
    ".idea",
    ".vscode",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRef {
    pub path: String,
    pub name: String,
    pub last_opened_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "config_schema_version", rename = "configSchemaVersion")]
    pub config_schema_version: u32,
    #[serde(flatten)]
    pub settings: Settings,
    pub settings_revision: u64,
    pub active_workspace: Option<String>,
    pub recent_workspaces: Vec<WorkspaceRef>,
    pub model_profiles: Vec<ModelProfileDto>,
    pub workspace_access: String,
    pub ignore_patterns: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut config = Self {
            config_schema_version: CONFIG_SCHEMA_VERSION,
            settings: Settings::default(),
            settings_revision: 1,
            active_workspace: None,
            recent_workspaces: Vec::new(),
            model_profiles: Vec::new(),
            workspace_access: "rw".into(),
            ignore_patterns: DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect(),
        };
        config.ensure_defaults();
        config
    }
}

impl AppConfig {
    pub fn load(config_dir: &Path) -> AppResult<Self> {
        let path = config_dir.join("config.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        let mut config = serde_json::from_str::<Self>(&text)
            .map_err(|e| AppError::parse(format!("应用配置损坏 {}: {e}", path.display())))?;
        config.config_schema_version = CONFIG_SCHEMA_VERSION;
        config.ensure_defaults();
        Ok(config)
    }

    pub fn save(&self, config_dir: &Path) -> AppResult<()> {
        std::fs::create_dir_all(config_dir)?;
        let path = config_dir.join("config.json");
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    pub fn ensure_defaults(&mut self) {
        if self.workspace_access.trim().is_empty() {
            self.workspace_access = "rw".into();
        }
        if self.ignore_patterns.is_empty() {
            self.ignore_patterns = DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect();
        }
        if self.model_profiles.is_empty() {
            let profile = ModelProfileDto {
                id: new_id(),
                revision: 1,
                data: mock_profile(),
            };
            self.model_profiles.push(profile);
        }
        let mock_id = self
            .model_profiles
            .iter()
            .find(|profile| profile.data.provider == "mock")
            .or_else(|| self.model_profiles.first())
            .map(|profile| profile.id.clone());
        if let Some(id) = mock_id {
            if self.settings.model_assignments.chat.is_none() {
                self.settings.model_assignments.chat = Some(id.clone());
            }
            if self.settings.model_assignments.reasoning.is_none() {
                self.settings.model_assignments.reasoning = Some(id.clone());
            }
            if self.settings.model_assignments.title.is_none() {
                self.settings.model_assignments.title = Some(id.clone());
            }
            if self.settings.model_assignments.summary.is_none() {
                self.settings.model_assignments.summary = Some(id);
            }
        }
        if self.settings_revision == 0 {
            self.settings_revision = 1;
        }
    }

    pub fn touch_workspace(&mut self, path: &Path, name: &str, at: i64) {
        let text = path.to_string_lossy().to_string();
        self.active_workspace = Some(text.clone());
        self.recent_workspaces.retain(|item| item.path != text);
        self.recent_workspaces.insert(
            0,
            WorkspaceRef {
                path: text,
                name: name.to_string(),
                last_opened_at: at,
            },
        );
        self.recent_workspaces.truncate(12);
    }

    pub fn workspace_root(&self) -> PathBuf {
        self.active_workspace
            .as_ref()
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
            .unwrap_or_else(paths::default_workspace)
    }

    pub fn model_profile(&self, id: &str) -> AppResult<ModelProfileDto> {
        self.model_profiles
            .iter()
            .find(|profile| profile.id == id)
            .cloned()
            .ok_or_else(|| AppError::not_found("model profile"))
    }

    pub fn save_model_profile(&mut self, profile: ModelProfileDto) {
        if let Some(existing) = self
            .model_profiles
            .iter_mut()
            .find(|item| item.id == profile.id)
        {
            *existing = profile;
        } else {
            self.model_profiles.push(profile);
        }
    }

    pub fn delete_model_profile(&mut self, id: &str) -> AppResult<()> {
        if !self.model_profiles.iter().any(|profile| profile.id == id) {
            return Err(AppError::not_found("model profile"));
        }
        self.model_profiles.retain(|profile| profile.id != id);
        let fallback = self.model_profiles.first().map(|profile| profile.id.clone());
        for slot in [
            &mut self.settings.model_assignments.chat,
            &mut self.settings.model_assignments.reasoning,
            &mut self.settings.model_assignments.title,
            &mut self.settings.model_assignments.summary,
        ] {
            if slot.as_deref() == Some(id) {
                *slot = fallback.clone();
            }
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_keeps_flat_settings_and_seeds_mock() {
        let config = AppConfig::default();
        assert!(config.config_schema_version > 0);
        assert!(!config.model_profiles.is_empty());
        assert!(config.settings.model_assignments.chat.is_some());
        let text = serde_json::to_string(&config).expect("serialize");
        let value = serde_json::from_str::<serde_json::Value>(&text).expect("json");
        assert!(value.get("configSchemaVersion").is_some());
        assert!(value.get("schemaVersion").is_some());
        let roundtrip = serde_json::from_str::<AppConfig>(&text).expect("deserialize");
        assert_eq!(roundtrip.config_schema_version, config.config_schema_version);
        assert_eq!(
            roundtrip.settings.theme_id,
            config.settings.theme_id
        );
        assert!(!roundtrip.model_profiles.is_empty());
    }
}
