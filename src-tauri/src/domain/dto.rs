use crate::domain::model::{now_ms, CardSurface, NodeType};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingState {
    pub completed: bool,
    pub theme_done: bool,
    pub language_done: bool,
    pub model_done: bool,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            completed: false,
            theme_done: false,
            language_done: false,
            model_done: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LayoutSettings {
    pub density: String,
    pub card_width: u32,
    pub show_tree: bool,
    pub sidebar_collapsed: bool,
    pub reduced_motion: bool,
    pub force_2d_tree: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            density: "comfortable".into(),
            card_width: 760,
            show_tree: true,
            sidebar_collapsed: false,
            reduced_motion: false,
            force_2d_tree: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelAssignments {
    pub chat: Option<String>,
    pub reasoning: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoBehavior {
    pub auto_title: bool,
    pub checkpoint_ms: u64,
    pub confirm_delete: bool,
    pub default_search: bool,
}

impl Default for AutoBehavior {
    fn default() -> Self {
        Self {
            auto_title: true,
            checkpoint_ms: 450,
            confirm_delete: true,
            default_search: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EditPermissions {
    pub read_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    pub network_targets_visible: bool,
    pub custom_base_url_ack: Vec<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            network_targets_visible: true,
            custom_base_url_ack: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub theme_id: String,
    pub locale: String,
    pub onboarding: OnboardingState,
    pub layout: LayoutSettings,
    pub model_assignments: ModelAssignments,
    pub auto_behavior: AutoBehavior,
    pub edit_permissions: EditPermissions,
    pub privacy: PrivacySettings,
    pub shortcuts: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut shortcuts = BTreeMap::new();
        shortcuts.insert("commandPalette".into(), "Ctrl+K".into());
        shortcuts.insert("search".into(), "Ctrl+Shift+F".into());
        shortcuts.insert("newProject".into(), "Ctrl+N".into());
        shortcuts.insert("send".into(), "Ctrl+Enter".into());
        shortcuts.insert("newline".into(), "Enter".into());
        shortcuts.insert("settings".into(), "Ctrl+,".into());
        shortcuts.insert("stop".into(), "Esc".into());
        shortcuts.insert("nextCard".into(), "Alt+ArrowRight".into());
        shortcuts.insert("prevCard".into(), "Alt+ArrowLeft".into());
        shortcuts.insert("rootCard".into(), "Alt+Home".into());
        Self {
            schema_version: 1,
            theme_id: "ink".into(),
            locale: "zh-CN".into(),
            onboarding: OnboardingState::default(),
            layout: LayoutSettings::default(),
            model_assignments: ModelAssignments::default(),
            auto_behavior: AutoBehavior::default(),
            edit_permissions: EditPermissions::default(),
            privacy: PrivacySettings::default(),
            shortcuts,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsState {
    pub settings: Settings,
    pub revision: u64,
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub vision: bool,
    pub tools: bool,
    pub json: bool,
    pub reasoning: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfileData {
    pub provider: String,
    pub label: String,
    pub base_url: String,
    pub model: String,
    pub context_window: u32,
    pub max_output: u32,
    pub capabilities: ModelCapabilities,
    pub pricing: ModelPricing,
    pub roles: Vec<String>,
    pub is_builtin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfileDto {
    pub id: String,
    pub revision: u64,
    pub data: ModelProfileData,
}

pub type ModelProfileInput = ModelProfileData;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfilePatch {
    pub label: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub context_window: Option<u32>,
    pub max_output: Option<u32>,
    pub capabilities: Option<ModelCapabilities>,
    pub pricing: Option<ModelPricing>,
    pub roles: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub message: String,
    pub model: String,
    pub model_count: Option<usize>,
}

// ---------------------------------------------------------------------------
// Project / folder
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub folder_id: Option<String>,
    pub favorite: bool,
    pub archived: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
    pub root_card_id: Option<String>,
    pub card_count: usize,
    pub message_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: ProjectSummary,
    pub root_card_id: String,
    pub default_model_profile_id: Option<String>,
    pub read_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FolderSummary {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub order: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateInput {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub folder_id: Option<Option<String>>,
    pub favorite: Option<bool>,
    pub archived: Option<bool>,
    pub default_model_profile_id: Option<Option<String>>,
    pub read_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreateInput {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FolderPatch {
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// Graph / card
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
    pub id: String,
    pub title: String,
    pub surface_kind: CardSurface,
    pub position: Option<[f64; 3]>,
    pub favorite: bool,
    pub locked: bool,
    pub archived: bool,
    pub parent_id: Option<String>,
    pub source_card_id: Option<String>,
    pub source_message_id: Option<String>,
    pub branch_point_message_id: Option<String>,
    pub child_count: usize,
    pub message_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EdgeView {
    pub id: String,
    pub from: String,
    pub to: String,
    pub relation: String,
    pub order: i64,
    pub metadata: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeRef {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub favorite: bool,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphView {
    pub project_id: String,
    pub root_card_id: Option<String>,
    pub cards: Vec<CardView>,
    pub edges: Vec<EdgeView>,
    pub documents: Vec<NodeRef>,
    pub knowledge: Vec<NodeRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardPathItem {
    pub id: String,
    pub title: String,
    pub surface_kind: CardSurface,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardDetail {
    pub card: CardView,
    pub path: Vec<CardPathItem>,
    pub turns: Vec<TurnDto>,
    pub default_model_profile_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardCreateInput {
    pub project_id: String,
    pub parent_card_id: String,
    pub relation: String,
    pub source_message_id: Option<String>,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CardPatch {
    pub title: Option<String>,
    pub position: Option<[f64; 3]>,
    pub locked: Option<bool>,
    pub favorite: Option<bool>,
    pub archived: Option<bool>,
}

// ---------------------------------------------------------------------------
// Turn / message
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub locator: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub currency: String,
    pub latency_ms: u64,
    pub estimated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDto {
    pub id: String,
    pub file_name: String,
    pub mime: String,
    pub size: u64,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
    pub id: String,
    pub turn_id: String,
    pub card_id: String,
    pub role: String,
    pub content: String,
    pub reasoning: String,
    pub citations: Vec<Citation>,
    pub attachments: Vec<AttachmentDto>,
    pub status: String,
    pub favorite: bool,
    pub edited_from: Option<String>,
    pub regenerated_from: Option<String>,
    pub usage: Option<Usage>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TurnDto {
    pub id: String,
    pub card_id: String,
    pub index: i64,
    pub collapsed: bool,
    pub model_profile_id: Option<String>,
    pub context_snapshot_id: Option<String>,
    pub messages: Vec<MessageDto>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TurnPatch {
    pub collapsed: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessagePatch {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub favorite: Option<bool>,
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContextOptions {
    pub excluded_item_ids: Vec<String>,
    pub pinned_message_ids: Vec<String>,
    pub knowledge_ids: Vec<String>,
    pub document_chunk_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextItem {
    pub id: String,
    pub kind: String,
    pub source_id: String,
    pub title: String,
    pub tokens: u64,
    pub reason: String,
    pub included: bool,
    pub dropped_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextPreview {
    pub items: Vec<ContextItem>,
    pub total_tokens: u64,
    pub budget: u64,
    pub dropped_count: usize,
    pub model_profile_id: String,
    pub message_count: usize,
    pub system_prompt_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextPreviewInput {
    pub card_id: String,
    pub draft_content: String,
    pub model_profile_id: String,
    #[serde(default)]
    pub context: ContextOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshotDto {
    pub id: String,
    pub card_id: String,
    pub turn_id: Option<String>,
    pub model_profile_id: String,
    pub items: Vec<ContextItem>,
    pub total_tokens: u64,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatSendInput {
    pub project_id: String,
    pub card_id: String,
    pub content: String,
    pub model_profile_id: String,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(default)]
    pub context: ContextOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatRegenerateInput {
    pub message_id: String,
    pub model_profile_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatStart {    pub run_id: String,
    pub turn_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub context_snapshot_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChatEvent {
    Started {
        run_id: String,
        turn_id: String,
        user_message_id: String,
        assistant_message_id: String,
        context_snapshot_id: String,
    },
    Delta {
        run_id: String,
        message_id: String,
        text: String,
    },
    Reasoning {
        run_id: String,
        message_id: String,
        text: String,
    },
    Usage {
        run_id: String,
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
        currency: String,
        latency_ms: u64,
        estimated: bool,
    },
    Checkpoint {
        run_id: String,
        message_id: String,
        chars: usize,
    },
    Done {
        run_id: String,
        message_id: String,
        status: String,
    },
    Cancelled {
        run_id: String,
        message_id: String,
    },
    Error {
        run_id: String,
        code: String,
        message: String,
        hint: Option<String>,
        retryable: bool,
    },
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub node_id: String,
    pub node_type: String,
    pub project_id: Option<String>,
    pub project_name: String,
    pub card_id: Option<String>,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub query: String,
    pub hits: Vec<SearchHit>,
    pub took_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchInput {
    pub query: String,
    pub project_id: Option<String>,
    pub types: Option<Vec<String>>,
    pub favorite_only: Option<bool>,
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub file_name: String,
    pub mime: String,
    pub pages: u32,
    pub chunk_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChunkDto {
    pub id: String,
    pub index: usize,
    pub page: Option<u32>,
    pub heading: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetail {
    pub document: DocumentSummary,
    pub chunks: Vec<ChunkDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChunkMatch {
    pub chunk_id: String,
    pub page: Option<u32>,
    pub snippet: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentImportInput {
    pub path: String,
    pub project_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUrlInput {
    pub url: String,
    pub project_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Knowledge
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceRef {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub locator: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDto {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub body: String,
    pub category: String,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub source_refs: Vec<SourceRef>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCreateInput {
    pub project_id: Option<String>,
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub source_refs: Option<Vec<SourceRef>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgePatch {
    pub title: Option<String>,
    pub body: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub source_refs: Option<Vec<SourceRef>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeFromMessageInput {
    pub message_id: String,
    pub selection: String,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCheckInput {
    pub id: String,
    pub model_profile_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCheckResult {
    pub feedback: String,
    pub model_profile_id: String,
}

// ---------------------------------------------------------------------------
// Bootstrap / diagnostics / data
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub projects: usize,
    pub cards: usize,
    pub messages: usize,
    pub documents: usize,
    pub knowledge: usize,
    pub nodes: usize,
    pub edges: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub version: String,
    pub platform: String,
    pub data_dir: String,
    pub db_path: String,
    pub counts: Counts,
    pub keychain_available: bool,
    pub model_profiles: usize,
    pub recent_error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub version: String,
    pub vault: VaultInfo,
    pub settings: Settings,
    pub settings_revision: u64,
    pub projects: Vec<ProjectSummary>,
    pub folders: Vec<FolderSummary>,
    pub models: Vec<ModelProfileDto>,
    pub counts: Counts,
    pub keychain_available: bool,
    pub init_error: Option<AppError>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub format: String,
    pub imported_projects: usize,
    pub imported_nodes: usize,
    pub imported_edges: usize,
    pub remapped: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub id: String,
    pub label: String,
    pub created_at: i64,
    pub node_count: usize,
    pub edge_count: usize,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub deleted_at: i64,
    pub project_id: Option<String>,
    pub project_name: String,
    pub card_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportVaultInput {
    pub path: String,
    pub mode: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPathInput {
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExportInput {
    pub id: String,
    pub path: String,
    pub format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectImportInput {
    pub path: String,
    pub folder_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesExportInput {
    pub path: String,
    pub format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SampleInstallInput {
    pub locale: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn empty_counts() -> Counts {
    Counts {
        projects: 0,
        cards: 0,
        messages: 0,
        documents: 0,
        knowledge: 0,
        nodes: 0,
        edges: 0,
    }
}

pub fn usage_zero() -> Usage {
    Usage {
        input_tokens: 0,
        output_tokens: 0,
        cost: 0.0,
        currency: "USD".into(),
        latency_ms: 0,
        estimated: true,
    }
}

pub fn message_data(
    role: &str,
    status: &str,
    reasoning: &str,
    citations: Vec<Citation>,
    attachments: Vec<AttachmentDto>,
) -> Value {
    serde_json::json!({
        "role": role,
        "status": status,
        "reasoning": reasoning,
        "citations": citations,
        "attachments": attachments,
        "usage": Value::Null,
    })
}

pub fn now_marker() -> i64 {
    now_ms()
}
