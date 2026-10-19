use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
            retryable: false,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn retryable(mut self, value: bool) -> Self {
        self.retryable = value;
        self
    }

    pub fn not_found(what: impl fmt::Display) -> Self {
        Self::new("not_found", format!("未找到：{what}"))
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }

    pub fn revision_conflict(expected: u64, actual: u64) -> Self {
        Self::new(
            "revision_conflict",
            format!("数据已被其他操作修改（期望 revision {expected}，实际 {actual}）"),
        )
        .with_hint("请刷新后重试。")
    }

    pub fn store(message: impl Into<String>) -> Self {
        Self::new("store_error", message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new("io_error", message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("unsupported", message)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new("parse_error", message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::parse(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
