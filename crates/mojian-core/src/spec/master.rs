//! SPEC 主副本定位、首次落地与权威 version / hash。
//!
//! 占位 SPEC 主副本树经 `include_dir` 编译进二进制（源在 `mojian-cli/assets/spec/`）。
//! 首次运行发现 `<data_dir>/spec/` 缺失时把嵌入树写出，形成客户端权威主副本。
//! 权威 `version` 读自主副本 `spec.toml`；权威 hash = 部署载荷树（除 `spec.toml`）的 tree hash。

use std::path::Path;

use include_dir::Dir;
use serde::Deserialize;

use crate::error::CoreError;
use crate::spec::hash::tree_hash_excluding;

/// 编译进二进制的占位 SPEC 主副本载荷树。
pub static EMBEDDED_SPEC: Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../mojian-cli/assets/spec");

/// 主副本版本元文件名（承载 `version`，不属部署载荷）。
const SPEC_META_FILE: &str = "spec.toml";

#[derive(Debug, Deserialize)]
struct SpecMeta {
    version: String,
}

/// 返回编译期嵌入的 SPEC 主副本载荷树引用。
pub fn embedded_spec() -> &'static Dir<'static> {
    &EMBEDDED_SPEC
}

/// bootstrap：`master_dir` 缺失时把嵌入骨架整棵写出，形成权威主副本；已存在则不动。
pub fn ensure_master(embedded: &Dir<'_>, master_dir: impl AsRef<Path>) -> Result<(), CoreError> {
    let master = master_dir.as_ref();
    if master.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(master).map_err(|source| CoreError::Io {
        path: master.to_path_buf(),
        source,
    })?;
    write_embedded(embedded, master)
}

fn write_embedded(dir: &Dir<'_>, dest_root: &Path) -> Result<(), CoreError> {
    for file in dir.files() {
        let dest = dest_root.join(file.path());
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|source| CoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(&dest, file.contents()).map_err(|source| CoreError::Io {
            path: dest.clone(),
            source,
        })?;
    }
    for sub in dir.dirs() {
        write_embedded(sub, dest_root)?;
    }
    Ok(())
}

/// 读主副本 `spec.toml` 的 `version` 作权威 spec 版本。
pub fn authoritative_version(master_dir: impl AsRef<Path>) -> Result<String, CoreError> {
    let path = master_dir.as_ref().join(SPEC_META_FILE);
    let text = std::fs::read_to_string(&path).map_err(|source| CoreError::Io {
        path: path.clone(),
        source,
    })?;
    let meta: SpecMeta = toml::from_str(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        .map_err(|source| CoreError::Io { path, source })?;
    Ok(meta.version)
}

/// 权威部署 hash = 对主副本部署载荷树（除 `spec.toml`）算 blake3 tree hash。
pub fn authoritative_hash(master_dir: impl AsRef<Path>) -> Result<String, CoreError> {
    tree_hash_excluding(master_dir.as_ref(), &[SPEC_META_FILE])
}
