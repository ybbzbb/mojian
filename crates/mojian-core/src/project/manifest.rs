//! 项目身份标记 `mojian.toml` 的读写。
//!
//! `mojian.toml` 落在项目目录根，是项目 → 中央 DB 的身份指针：`project_id` 定位机器状态，
//! `spec_version` 供人工核对已部署 SPEC 版本。字段最小，人可读。

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// 项目根 `mojian.toml` 的文件名。
const MANIFEST_FILE: &str = "mojian.toml";

/// `mojian.toml` 的内容模型：项目身份标记。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project_id: String,
    pub spec_version: String,
}

/// 将 manifest 序列化写入 `<dir>/mojian.toml`（覆盖同名文件）。
pub fn write_manifest(dir: impl AsRef<Path>, manifest: &ProjectManifest) -> Result<(), CoreError> {
    let path = dir.as_ref().join(MANIFEST_FILE);
    let text = toml::to_string(manifest)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .map_err(|source| CoreError::Io {
            path: path.clone(),
            source,
        })?;
    std::fs::write(&path, text).map_err(|source| CoreError::Io { path, source })
}

/// 读回 `<dir>/mojian.toml`。缺文件（`NotFound`）或内容非法均返回 `CoreError`。
pub fn read_manifest(dir: impl AsRef<Path>) -> Result<ProjectManifest, CoreError> {
    let path = dir.as_ref().join(MANIFEST_FILE);
    let text = std::fs::read_to_string(&path).map_err(|source| CoreError::Io {
        path: path.clone(),
        source,
    })?;
    toml::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .map_err(|source| CoreError::Io { path, source })
}
