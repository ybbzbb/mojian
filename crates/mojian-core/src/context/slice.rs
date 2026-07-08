//! 切片：整文件 / 段级（`#anchor` 稳定小标题抽取）双粒度 + 内容 blake3 hash。
//!
//! 段级切片依赖 SSOT 的稳定小标题锚点：从匹配 `anchor` 的 markdown 标题起，抽取到下一个
//! 同级或更高级标题（不含）之间的命名段落。内容 hash 与 `artifact_ref.content_hash` /
//! generation.jsonl 输入切片 hash 同源，供后续过期检测与 token 对账。

use std::path::Path;

use crate::error::CoreError;

/// 单条切片结果：项目相对路径（`/` 分隔）、可选段级锚点、切片内容及其 blake3 hex hash。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice {
    pub rel_path: String,
    pub anchor: Option<String>,
    pub content: String,
    pub content_hash: String,
}

/// 计算内容的 blake3 hex hash（复用既有 `blake3` 依赖）。
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// 把相对路径规范化为以 `/` 分隔的字符串（跨平台确定性）。
fn rel_string(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// 读取 `project_dir/rel_path` 并按 `anchor` 切片：`None` 取整文件，`Some(a)` 取段级。
/// 文件缺失返回 `CoreError::Io`；段级锚点未命中返回 `CoreError::SymbolUnresolved`。
pub fn slice_ref(project_dir: &Path, rel_path: &Path, anchor: Option<&str>) -> Result<Slice, CoreError> {
    let abs = project_dir.join(rel_path);
    let text = std::fs::read_to_string(&abs).map_err(|source| CoreError::Io {
        path: abs.clone(),
        source,
    })?;

    let content = match anchor {
        None => text,
        Some(a) => extract_section(&text, a).ok_or_else(|| CoreError::SymbolUnresolved {
            symbol: format!("{}#{a}", rel_string(rel_path)),
            reason: "段级锚点小标题未命中".to_string(),
        })?,
    };

    let content_hash = content_hash(content.as_bytes());
    Ok(Slice {
        rel_path: rel_string(rel_path),
        anchor: anchor.map(str::to_string),
        content,
        content_hash,
    })
}

/// markdown 标题层级：返回前导 `#` 个数（须后接空白或行尾），否则 `None`（如 `#tag`）。
fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    let after = &trimmed[level..];
    if after.is_empty() || after.starts_with(char::is_whitespace) {
        Some(level)
    } else {
        None
    }
}

/// 标题正文（去掉前导 `#` 与两侧空白）。
fn heading_text(line: &str) -> String {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    trimmed[level..].trim().to_string()
}

/// 轻量 slug：小写、字母数字保留、空白/`-`/`_` 归一为 `-`，其余丢弃。
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c == ' ' || c == '-' || c == '_' || c == '\t' {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// 抽取名为 `anchor` 的小标题段落（标题正文精确匹配或 slug 匹配），从标题行起到下一个
/// 同级或更高级标题（不含）之间。未命中返回 `None`。
fn extract_section(text: &str, anchor: &str) -> Option<String> {
    let anchor = anchor.trim();
    let lines: Vec<&str> = text.lines().collect();

    let mut start = None;
    let mut level = 0;
    for (i, line) in lines.iter().enumerate() {
        if let Some(l) = heading_level(line) {
            let ht = heading_text(line);
            if ht == anchor || slugify(&ht) == slugify(anchor) {
                start = Some(i);
                level = l;
                break;
            }
        }
    }
    let start = start?;

    let mut end = lines.len();
    for (offset, line) in lines[start + 1..].iter().enumerate() {
        if let Some(l) = heading_level(line) {
            if l <= level {
                end = start + 1 + offset;
                break;
            }
        }
    }

    Some(lines[start..end].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp(name: &str, rel: &str, body: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "mojian-slice-test-{}-{}",
            std::process::id(),
            name
        ));
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        (dir, PathBuf::from(rel))
    }

    #[test]
    fn whole_file_slice_hash_is_stable() {
        let (dir, rel) = write_temp("whole", "a.md", "# 标题\n正文内容\n");
        let s1 = slice_ref(&dir, &rel, None).unwrap();
        let s2 = slice_ref(&dir, &rel, None).unwrap();
        assert_eq!(s1.content, "# 标题\n正文内容\n");
        assert_eq!(s1.content_hash, s2.content_hash, "同内容整文件 hash 应稳定");
        assert!(s1.anchor.is_none());
        assert_eq!(s1.content_hash, content_hash(s1.content.as_bytes()));
    }

    #[test]
    fn section_slice_extracts_named_anchor_only() {
        let body = "# 顶级\n序言\n\n## inputs\n- 项 A\n- 项 B\n\n## output\n产出说明\n";
        let (dir, rel) = write_temp("section", "spec.md", body);
        let s = slice_ref(&dir, &rel, Some("inputs")).unwrap();
        assert!(s.content.contains("## inputs"));
        assert!(s.content.contains("项 A"));
        assert!(s.content.contains("项 B"));
        // 段级边界：不越界到下一个同级标题。
        assert!(!s.content.contains("## output"));
        assert!(!s.content.contains("产出说明"));
        assert!(!s.content.contains("序言"));
        assert_eq!(s.anchor.as_deref(), Some("inputs"));
    }

    #[test]
    fn section_boundary_stops_at_higher_level_heading() {
        let body = "# A\n## sub\n子段内容\n# B\nB 段\n";
        let (dir, rel) = write_temp("boundary", "b.md", body);
        let s = slice_ref(&dir, &rel, Some("sub")).unwrap();
        assert!(s.content.contains("子段内容"));
        // 遇更高级标题 `# B` 即止。
        assert!(!s.content.contains("# B"));
        assert!(!s.content.contains("B 段"));
    }

    #[test]
    fn section_anchor_miss_is_unresolved() {
        let (dir, rel) = write_temp("miss", "c.md", "# 只有这个\n内容\n");
        let err = slice_ref(&dir, &rel, Some("不存在")).unwrap_err();
        assert!(matches!(err, CoreError::SymbolUnresolved { .. }));
        // 直接测抽取函数：未命中返回 None。
        assert!(extract_section("# x\n", "y").is_none());
    }

    #[test]
    fn missing_file_returns_io_error() {
        let dir = std::env::temp_dir().join(format!("mojian-slice-missing-{}", std::process::id()));
        let err = slice_ref(&dir, &PathBuf::from("nope.md"), None).unwrap_err();
        assert!(matches!(err, CoreError::Io { .. }));
    }

    #[test]
    fn hash_prefix_style_lines_are_not_headings() {
        // `#tag`（无空白）不是标题，不应误命中。
        assert!(heading_level("#tag").is_none());
        assert_eq!(heading_level("## 标题"), Some(2));
        assert_eq!(heading_level("# 顶"), Some(1));
    }
}
