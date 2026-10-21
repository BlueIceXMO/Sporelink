use std::path::PathBuf;

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

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

/// Application-level configuration. Secrets stay in the OS keychain.
pub fn config_dir() -> PathBuf {
    if let Ok(value) = std::env::var("SPORELINK_CONFIG_DIR") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    home_dir().join(".sporelink")
}

fn documents_dir() -> PathBuf {
    dirs::document_dir().unwrap_or_else(|| home_dir().join("Documents"))
}

fn push_unique(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|item| item == &path) {
        candidates.push(path);
    }
}

/// Candidate roots for the default workspace. The first one is the
/// documented default; the rest are writability fallbacks for redirected or
/// restricted Documents folders.
pub fn default_workspace_candidates() -> Vec<PathBuf> {
    if let Ok(value) = std::env::var("SPORELINK_WORKSPACE") {
        if !value.trim().is_empty() {
            return vec![PathBuf::from(value)];
        }
    }
    let mut candidates = Vec::new();
    push_unique(&mut candidates, documents_dir().join("Sporelink"));
    push_unique(&mut candidates, home_dir().join("Documents").join("Sporelink"));
    push_unique(&mut candidates, home_dir().join("Sporelink"));
    push_unique(&mut candidates, exe_dir().join("SporelinkWorkspace"));
    push_unique(
        &mut candidates,
        std::env::temp_dir().join("SporelinkWorkspace"),
    );
    candidates
}

/// Default workspace: Documents/Sporelink. A debug/test override can point
/// somewhere else via SPORELINK_WORKSPACE.
pub fn default_workspace() -> PathBuf {
    default_workspace_candidates()
        .into_iter()
        .next()
        .unwrap_or_else(|| home_dir().join("Sporelink"))
}

/// WebView2 profile lives with the app config, not in a workspace and not in
/// Windows AppData.
pub fn webview_dir() -> PathBuf {
    if let Ok(value) = std::env::var("WEBVIEW2_USER_DATA_FOLDER") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    let mut candidates = vec![config_dir().join("webview2")];
    candidates.push(exe_dir().join(".sporelink-webview2"));
    first_writable(candidates, "SporelinkWebView2")
}

pub fn ensure_webview_env() {
    if std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").is_none() {
        let dir = webview_dir();
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", dir);
    }
}

pub fn ensure_tauri_dirs() {
    let _ = std::fs::create_dir_all(config_dir());
}

pub fn log_dir() -> PathBuf {
    let mut candidates = vec![config_dir().join("logs")];
    candidates.push(exe_dir().join("logs"));
    first_writable(candidates, "SporelinkLogs")
}

pub fn append_log(message: &str) {
    let dir = log_dir();
    let path = dir.join("startup.log");
    let line = format!("[{}] {}\n", time_stamp(), message.replace('\n', " "));
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
