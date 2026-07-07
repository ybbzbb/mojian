//! 部署载荷树的 blake3 tree hash。
//!
//! tree hash 定义：对目录内每个文件按「相对路径升序」拼接「相对路径 + 该文件内容的
//! blake3 hex」，再对整体拼接结果算 blake3，得确定性 hex（顺序无关、内容敏感）。
//! 用于 SPEC 部署缓存的一致性比对，不追求与外部标准互操作。

use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// 递归收集 `dir` 下所有普通文件的绝对路径（不含目录本身）。
pub(crate) fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), CoreError> {
    let read = std::fs::read_dir(dir).map_err(|source| CoreError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in read {
        let entry = entry.map_err(|source| CoreError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| CoreError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_files(&path, out)?;
        } else if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// 将 `file` 相对 `root` 的路径规范化为以 `/` 分隔的字符串（跨平台确定性）。
fn rel_string(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// 对给定文件集合（相对 `root` 求相对路径）计算 tree hash，跳过相对路径落在
/// `exclude_rel` 中的文件。
pub(crate) fn hash_files(
    root: &Path,
    files: &[PathBuf],
    exclude_rel: &[&str],
) -> Result<String, CoreError> {
    let mut entries: Vec<(String, &PathBuf)> = files
        .iter()
        .map(|f| (rel_string(root, f), f))
        .filter(|(rel, _)| !exclude_rel.contains(&rel.as_str()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = blake3::Hasher::new();
    for (rel, abs) in &entries {
        let content = std::fs::read(abs).map_err(|source| CoreError::Io {
            path: (*abs).clone(),
            source,
        })?;
        let content_hash = blake3::hash(&content);
        hasher.update(rel.as_bytes());
        hasher.update(content_hash.to_hex().as_bytes());
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// 对 `dir` 下整棵文件树计算 tree hash。
pub fn tree_hash(dir: impl AsRef<Path>) -> Result<String, CoreError> {
    let root = dir.as_ref();
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    hash_files(root, &files, &[])
}

/// 对 `dir` 下文件树计算 tree hash，但排除相对路径命中 `exclude_rel` 的文件
/// （用于主副本载荷 hash：排除 `spec.toml`）。
pub(crate) fn tree_hash_excluding(dir: &Path, exclude_rel: &[&str]) -> Result<String, CoreError> {
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    hash_files(dir, &files, exclude_rel)
}
