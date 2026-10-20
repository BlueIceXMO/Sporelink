use std::path::PathBuf;

const APP_DIR: &str = "Sporelink";
const TAURI_IDENTIFIER: &str = "com.sporelink.desktop";

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn first_writable(candidates: Vec<PathBuf>, fallback_name: &str) -> PathBuf {
    for candidate in &candidates {
        if std::fs::create_dir_all(candidate).is_ok() {
            return candidate.clone();
        }
    }
    let fallback = std::env::temp_dir().join(fallback_name);
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

/// Business data directory. Prefer `SPORELINK_DATA_DIR`, then the OS data dir,
/// then a portable folder next to the executable, then temp.
pub fn data_dir() -> PathBuf {
    if let Ok(value) = std::env::var("SPORELINK_DATA_DIR") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    let mut candidates = Vec::new();
    if let Some(dir) = dirs::data_dir() {
        candidates.push(dir.join(APP_DIR));
    }
    if let Some(dir) = dirs::data_local_dir() {
        candidates.push(dir.join(APP_DIR));
    }
    candidates.push(exe_dir().join("SporelinkData"));
    first_writable(candidates, "SporelinkData")
}

/// WebView2 profile directory. Tauri's default path is based on the bundle
/// identifier and can be blocked by restrictive Windows folders; an explicit
/// profile avoids a silent "app does not start" failure.
pub fn webview_dir() -> PathBuf {
    if let Ok(value) = std::env::var("WEBVIEW2_USER_DATA_FOLDER") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    let mut candidates = Vec::new();
    if let Some(dir) = dirs::data_local_dir() {
        candidates.push(dir.join(APP_DIR).join("WebView2"));
    }
    candidates.push(data_dir().join("WebView2"));
    candidates.push(exe_dir().join("WebView2"));
    first_writable(candidates, "SporelinkWebView2")
}

pub fn ensure_webview_env() {
    if std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").is_none() {
        let dir = webview_dir();
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", dir);
    }
}

/// Best-effort creation of the locations Tauri/plugins may touch by identifier.
pub fn ensure_tauri_dirs() {
    let mut dirs = Vec::new();
    if let Some(dir) = dirs::config_dir() {
        dirs.push(dir.join(TAURI_IDENTIFIER));
    }
    if let Some(dir) = dirs::data_local_dir() {
        dirs.push(dir.join(TAURI_IDENTIFIER));
    }
    if let Some(dir) = dirs::data_dir() {
        dirs.push(dir.join(TAURI_IDENTIFIER));
    }
    for dir in dirs {
        let _ = std::fs::create_dir_all(dir);
    }
}

pub fn log_dir() -> PathBuf {
    let mut candidates = Vec::new();
    if let Some(dir) = dirs::data_local_dir() {
        candidates.push(dir.join(APP_DIR).join("logs"));
    }
    candidates.push(data_dir().join("logs"));
    candidates.push(exe_dir().join("logs"));
    first_writable(candidates, "SporelinkLogs")
}

pub fn append_log(message: &str) {
    let dir = log_dir();
    let path = dir.join("startup.log");
    let line = format!(
        "[{}] {}\n",
        time_stamp(),
        message.replace('\n', " ")
    );
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
    }
}

fn time_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    millis.to_string()
}
