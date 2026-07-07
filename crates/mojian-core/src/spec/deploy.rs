//! SPEC 部署与打开时的 hash 覆盖（选项 A：项目内 SPEC 为纯可弃缓存）。
//!
//! 部署：先删除项目内旧部署目标，再把主副本载荷树（除 `spec.toml`）1:1 递归拷入项目根。
//! 打开：实时重算项目实际部署树的 tree hash 与客户端权威 hash 比对，不一致则覆盖重部署。
//! 仅操作部署目标条目，不触及客户端主副本（SSOT）。

use std::path::Path;

use crate::error::CoreError;
use crate::spec::hash::{collect_files, hash_files, tree_hash_excluding};
use crate::spec::master::authoritative_version;

/// 主副本中不属于部署载荷的版本元文件名。
const SPEC_META_FILE: &str = "spec.toml";

/// 项目内的部署目标条目（对齐 engine.md 项目文件布局）。
const DEPLOY_TARGETS: &[&str] = &[".claude/agents", ".claude/skills", "CLAUDE.md", "prompts"];

/// 把 `payload_src`（主副本目录）的部署载荷树拷入 `project_dir`，返回 `(version, hash)`。
/// 拷贝前先删除项目内旧部署目标，保证删除类变更也被覆盖。
pub fn deploy_spec(payload_src: &Path, project_dir: &Path) -> Result<(String, String), CoreError> {
    for target in DEPLOY_TARGETS {
        remove_target(&project_dir.join(target))?;
    }
    copy_payload(payload_src, project_dir)?;
    let version = authoritative_version(payload_src)?;
    let hash = tree_hash_excluding(payload_src, &[SPEC_META_FILE])?;
    Ok((version, hash))
}

/// 打开项目时的漂移自愈：重算项目实际部署树 hash 与权威 hash 比对。
/// 不一致 → 覆盖重部署，返回 `(true, new_hash)`；一致 → 不写，返回 `(false, current_hash)`。
pub fn sync_if_drifted(
    project_dir: &Path,
    authoritative_hash: &str,
    payload_src: &Path,
) -> Result<(bool, String), CoreError> {
    let current = deployed_tree_hash(project_dir)?;
    if current == authoritative_hash {
        Ok((false, current))
    } else {
        let (_version, new_hash) = deploy_spec(payload_src, project_dir)?;
        Ok((true, new_hash))
    }
}

fn remove_target(path: &Path) -> Result<(), CoreError> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(CoreError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    let result = if meta.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_payload(src: &Path, dest: &Path) -> Result<(), CoreError> {
    let mut files = Vec::new();
    collect_files(src, &mut files)?;
    for abs in files {
        let rel = match abs.strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        if rel == Path::new(SPEC_META_FILE) {
            continue;
        }
        let out = dest.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|source| CoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::copy(&abs, &out).map_err(|source| CoreError::Io {
            path: out.clone(),
            source,
        })?;
    }
    Ok(())
}

/// 只对项目内部署目标条目求 tree hash（相对 `project_dir`），与权威 hash 同构可比。
fn deployed_tree_hash(project_dir: &Path) -> Result<String, CoreError> {
    let mut files = Vec::new();
    for target in DEPLOY_TARGETS {
        let path = project_dir.join(target);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => collect_files(&path, &mut files)?,
            Ok(meta) if meta.is_file() => files.push(path),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(CoreError::Io { path, source }),
        }
    }
    hash_files(project_dir, &files, &[])
}
