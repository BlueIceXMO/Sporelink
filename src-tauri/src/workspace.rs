use crate::domain::dto::{WorkspaceFile, WorkspaceInfo};
use crate::error::{AppError, AppResult};
use crate::paths;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};

pub const WORKSPACE_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    ReadWrite,
    ReadOnly,
    Full,
}

impl AccessMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadWrite => "rw",
            Self::ReadOnly => "ro",
            Self::Full => "full",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "ro" => Self::ReadOnly,
            "full" => Self::Full,
            _ => Self::ReadWrite,
        }
    }
}

pub struct Workspace {
    pub root: PathBuf,
    pub internal: PathBuf,
    pub manifest: WorkspaceManifest,
    pub access: AccessMode,
}

impl Workspace {
    pub fn open_or_create(root: &Path, access: AccessMode) -> AppResult<(Self, bool)> {
        if root.as_os_str().is_empty() {
            return Err(AppError::invalid("工作区路径不能为空"));
        }
        if root
            .components()
            .any(|component| matches!(component, Component::Normal(name) if name == ".spore"))
        {
            return Err(AppError::invalid("不能把 .spore 目录当作工作区"));
        }
        std::fs::create_dir_all(root).map_err(|e| {
            AppError::io(format!("创建工作区失败 {}: {e}", root.display()))
        })?;
        let root = dunce::canonicalize(&root)
            .map_err(|e| AppError::io(format!("解析工作区路径失败 {}: {e}", root.display())))?;
        let internal = root.join(".spore");
        std::fs::create_dir_all(&internal)?;
        for sub in ["db", "snapshots", "logs"] {
            std::fs::create_dir_all(internal.join(sub))?;
        }
        let manifest_path = internal.join("workspace.json");
        let (manifest, created) = if manifest_path.exists() {
            let text = std::fs::read_to_string(&manifest_path)?;
            let manifest = serde_json::from_str::<WorkspaceManifest>(&text)
                .map_err(|e| AppError::parse(format!("工作区清单损坏 {}: {e}", manifest_path.display())))?;
            if manifest.schema_version > WORKSPACE_SCHEMA_VERSION {
                return Err(AppError::new(
                    "unsupported",
                    format!(
                        "工作区版本 {} 高于当前支持的 {}",
                        manifest.schema_version, WORKSPACE_SCHEMA_VERSION
                    ),
                ));
            }
            (manifest, false)
        } else {
            let name = root
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Sporelink".into());
            let manifest = WorkspaceManifest {
                schema_version: WORKSPACE_SCHEMA_VERSION,
                id: crate::domain::model::new_id(),
                name,
                created_at: crate::domain::model::now_ms(),
            };
            std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
            (manifest, true)
        };
        Ok((
            Self {
                root,
                internal,
                manifest,
                access,
            },
            created,
        ))
    }

    pub fn info(&self) -> WorkspaceInfo {
        WorkspaceInfo {
            id: self.manifest.id.clone(),
            name: self.manifest.name.clone(),
            root: self.root.to_string_lossy().to_string(),
            internal: self.internal.to_string_lossy().to_string(),
            access: self.access.as_str().into(),
            created_at: self.manifest.created_at,
        }
    }

    pub fn db_dir(&self) -> PathBuf {
        self.internal.join("db")
    }

    pub fn snapshot_dir(&self) -> PathBuf {
        self.internal.join("snapshots")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.internal.join("logs")
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.root.join("attachments")
    }

    pub fn is_internal(&self, path: &Path) -> bool {
        if path.starts_with(&self.internal) {
            return true;
        }
        path.components().any(|component| {
            matches!(component, Component::Normal(name)
                if name == ".spore" || name == ".sporelink")
        })
    }

    fn ensure_read_write(&self) -> AppResult<()> {
        if self.access != AccessMode::ReadWrite && self.access != AccessMode::Full {
            return Err(AppError::new("unsupported", "当前工作区是只读模式"));
        }
        Ok(())
    }

    pub fn relative_path(&self, path: &Path) -> AppResult<String> {
        let path = dunce::canonicalize(path)
            .map_err(|e| AppError::io(format!("文件不存在 {}: {e}", path.display())))?;
        if !path.starts_with(&self.root) {
            return Err(AppError::invalid("文件不在当前工作区"));
        }
        if self.is_internal(&path) {
            return Err(AppError::invalid(".spore 内部文件不能作为用户文件"));
        }
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| AppError::invalid("无法计算工作区相对路径"))?;
        Ok(relative.to_string_lossy().replace('\\', "/"))
    }

    pub fn resolve_user_file(&self, input: &str) -> AppResult<PathBuf> {
        let candidate = if Path::new(input).is_absolute() {
            PathBuf::from(input)
        } else {
            self.root.join(input)
        };
        let path = dunce::canonicalize(&candidate)
            .map_err(|e| AppError::io(format!("文件不存在 {}: {e}", candidate.display())))?;
        if !path.starts_with(&self.root) {
            return Err(AppError::invalid("文件不在当前工作区"));
        }
        if self.is_internal(&path) {
            return Err(AppError::invalid(".spore 内部文件不能作为用户文件"));
        }
        if !path.is_file() {
            return Err(AppError::invalid("目标不是文件"));
        }
        Ok(path)
    }

    pub fn read_text(
        &self,
        input: &str,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> AppResult<String> {
        let path = self.resolve_user_file(input)?;
        let metadata = std::fs::metadata(&path)?;
        if metadata.len() > 2_000_000 {
            return Err(AppError::invalid("文件超过 2MB，不能直接读取"));
        }
        let bytes = std::fs::read(&path)?;
        if bytes.contains(&0) {
            return Err(AppError::invalid("二进制文件不能按文本读取"));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| AppError::invalid("文件不是 UTF-8 文本"))?;
        let Some(start) = start_line else {
            return Ok(text);
        };
        let start = start.max(1);
        let end = end_line.unwrap_or(start);
        if end < start {
            return Err(AppError::invalid("结束行不能小于起始行"));
        }
        Ok(text
            .lines()
            .skip(start - 1)
            .take(end - start + 1)
            .collect::<Vec<_>>()
            .join("\n"))
    }

    pub fn copy_external(&self, source: &Path) -> AppResult<PathBuf> {
        self.ensure_read_write()?;
        let source = dunce::canonicalize(source)
            .map_err(|e| AppError::io(format!("文件不存在 {}: {e}", source.display())))?;
        if source.starts_with(&self.root) {
            if self.is_internal(&source) {
                return Err(AppError::invalid(".spore 内部文件不能导入"));
            }
            return Ok(source);
        }
        let bytes = std::fs::read(&source)?;
        let name = source
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".into());
        self.write_attachment(&name, &bytes)
    }

    pub fn write_attachment(&self, name: &str, bytes: &[u8]) -> AppResult<PathBuf> {
        self.ensure_read_write()?;
        let attachments = self.attachments_dir();
        std::fs::create_dir_all(&attachments)?;
        let hash = hex::encode(Sha256::digest(bytes));
        let safe: String = name
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                    ch
                } else {
                    '_'
                }
            })
            .collect();
        let safe = if safe.trim().is_empty() { "file" } else { &safe };
        let preferred = attachments.join(safe);
        let destination = if preferred.exists() {
            attachments.join(format!("{}-{}", &hash[..8], safe))
        } else {
            preferred
        };
        if !destination.exists() {
            std::fs::write(&destination, bytes)?;
        }
        Ok(destination)
    }

    pub fn list_user_files(&self, ignore: &[String]) -> AppResult<Vec<WorkspaceFile>> {
        let mut files = Vec::new();
        self.walk(&self.root, ignore, 0, &mut files)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    fn walk(
        &self,
        dir: &Path,
        ignore: &[String],
        depth: usize,
        out: &mut Vec<WorkspaceFile>,
    ) -> AppResult<()> {
        if depth > 12 || out.len() >= 5000 {
            return Ok(());
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if self.is_internal(&path) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if ignore.iter().any(|pattern| pattern == &name) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                self.walk(&path, ignore, depth + 1, out)?;
            } else if meta.is_file() {
                let relative = match path.strip_prefix(&self.root) {
                    Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
                    Err(_) => continue,
                };
                out.push(WorkspaceFile {
                    path: relative,
                    name,
                    size: meta.len(),
                    is_dir: false,
                    modified_at: meta
                        .modified()
                        .ok()
                        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|duration| duration.as_millis() as i64)
                        .unwrap_or(0),
                });
            }
        }
        Ok(())
    }

    pub fn default_ignore(&self) -> Vec<String> {
        crate::config::DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect()
    }

    pub fn default_root() -> PathBuf {
        paths::default_workspace()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_isolates_spore_and_copies_external_files() {
        let root = tempfile::tempdir().expect("root");
        let (workspace, created) =
            Workspace::open_or_create(root.path(), AccessMode::ReadWrite).expect("workspace");
        assert!(created);
        std::fs::create_dir_all(root.path().join("papers")).expect("papers");
        std::fs::write(root.path().join("papers").join("note.md"), "# Note").expect("note");
        std::fs::write(workspace.internal.join("secret.json"), "{}").expect("secret");

        let files = workspace.list_user_files(&[]).expect("files");
        assert!(files.iter().any(|file| file.path == "papers/note.md"));
        assert!(!files.iter().any(|file| file.path.contains(".spore")));
        assert!(workspace
            .resolve_user_file(".spore/secret.json")
            .is_err());
        assert_eq!(
            workspace.read_text("papers/note.md", None, None).expect("read"),
            "# Note"
        );

        let outside = tempfile::tempdir().expect("outside");
        let source = outside.path().join("outside.txt");
        std::fs::write(&source, "outside").expect("outside file");
        let copied = workspace.copy_external(&source).expect("copy");
        assert!(copied.starts_with(root.path().join("attachments")));
        assert!(source.exists());
        let files = workspace.list_user_files(&[]).expect("files");
        assert!(files.iter().any(|file| file.path.starts_with("attachments/")));
    }
}
