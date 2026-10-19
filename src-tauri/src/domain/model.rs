use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Project,
    Folder,
    Card,
    Turn,
    Message,
    Document,
    Chunk,
    Knowledge,
    Attachment,
    Settings,
    ModelProfile,
    Run,
    ContextSnapshot,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Folder => "folder",
            Self::Card => "card",
            Self::Turn => "turn",
            Self::Message => "message",
            Self::Document => "document",
            Self::Chunk => "chunk",
            Self::Knowledge => "knowledge",
            Self::Attachment => "attachment",
            Self::Settings => "settings",
            Self::ModelProfile => "model_profile",
            Self::Run => "run",
            Self::ContextSnapshot => "context_snapshot",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRelation {
    Contains,
    DerivesFrom,
    BranchOf,
    ParallelTo,
    Cites,
    References,
    AttachmentOf,
    Mentions,
    Summarizes,
}

impl EdgeRelation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::DerivesFrom => "derives_from",
            Self::BranchOf => "branch_of",
            Self::ParallelTo => "parallel_to",
            Self::Cites => "cites",
            Self::References => "references",
            Self::AttachmentOf => "attachment_of",
            Self::Mentions => "mentions",
            Self::Summarizes => "summarizes",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CardSurface {
    Root,
    Dive,
    Diverge,
    Branch,
    Child,
    Note,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub project_id: Option<String>,
    pub title: String,
    pub body: String,
    pub data: Value,
    pub position: Option<[f64; 3]>,
    pub favorite: bool,
    pub locked: bool,
    pub archived: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

impl Node {
    pub fn new(
        node_type: NodeType,
        project_id: Option<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        data: Value,
    ) -> Self {
        let now = now_ms();
        Self {
            id: new_id(),
            node_type,
            project_id,
            title: title.into(),
            body: body.into(),
            data,
            position: None,
            favorite: false,
            locked: false,
            archived: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
            revision: 1,
        }
    }

    pub fn is_live(&self) -> bool {
        self.deleted_at.is_none()
    }

    pub fn data_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(Value::as_str)
    }

    pub fn data_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key).and_then(Value::as_i64)
    }

    pub fn data_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(Value::as_bool)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub project_id: Option<String>,
    pub from: String,
    pub to: String,
    pub relation: EdgeRelation,
    pub order: i64,
    pub metadata: Value,
    pub created_at: i64,
    pub updated_at: i64,
    pub revision: u64,
}

impl Edge {
    pub fn new(
        project_id: Option<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation: EdgeRelation,
        order: i64,
        metadata: Value,
    ) -> Self {
        let now = now_ms();
        Self {
            id: new_id(),
            project_id,
            from: from.into(),
            to: to.into(),
            relation,
            order,
            metadata,
            created_at: now,
            updated_at: now,
            revision: 1,
        }
    }
}
