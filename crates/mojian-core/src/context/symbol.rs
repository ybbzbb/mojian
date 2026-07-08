//! 符号引用解析器（选型 5-A：手写小解析器，零依赖）。
//!
//! 文法 `<source>.<selector>[:{params}][#anchor]`（如 `bible.style#skeleton`、
//! `plan.chapters:{batch}`、`prev_skeleton:{ch-1}`、纯整文件 `workspace`）。按 `#` / `:` /
//! `.` 切分，再按当前状态代入 `{arc_id}` / `{batch}` / `{ch-1}` 等占位（支持 `名±N` 整数
//! 算术），最后经 source→基路径映射解析为项目相对路径 + 可选段级锚点。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// 符号解析结果：项目相对路径 + 可选段级锚点 + 已代入的参数串（供日志追溯）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRef {
    pub path: PathBuf,
    pub anchor: Option<String>,
    pub params: Option<String>,
}

/// 文法切分结果（未代入占位）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSymbol {
    source: String,
    selector: Option<String>,
    params: Option<String>,
    anchor: Option<String>,
}

fn unresolved(symbol: &str, reason: impl Into<String>) -> CoreError {
    CoreError::SymbolUnresolved {
        symbol: symbol.to_string(),
        reason: reason.into(),
    }
}

/// 按 `#` / `:` / `.` 切分符号，不做占位代入。
fn parse_symbol(raw: &str) -> Result<ParsedSymbol, CoreError> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(unresolved(raw, "空符号"));
    }

    let (head, anchor) = match s.split_once('#') {
        Some((h, a)) if !a.is_empty() => (h, Some(a.to_string())),
        Some((_, _)) => return Err(unresolved(raw, "`#` 后锚点为空")),
        None => (s, None),
    };

    let (head, params) = match head.split_once(':') {
        Some((h, p)) => {
            let inner = p
                .trim()
                .strip_prefix('{')
                .and_then(|x| x.strip_suffix('}'))
                .ok_or_else(|| unresolved(raw, "参数段必须形如 :{...}"))?;
            if inner.is_empty() {
                return Err(unresolved(raw, "参数段 {} 为空"));
            }
            (h, Some(inner.to_string()))
        }
        None => (head, None),
    };

    let (source, selector) = match head.split_once('.') {
        Some((so, se)) => {
            if se.is_empty() {
                return Err(unresolved(raw, "`.` 后 selector 为空"));
            }
            (so.to_string(), Some(se.to_string()))
        }
        None => (head.to_string(), None),
    };

    if source.is_empty() {
        return Err(unresolved(raw, "source 为空"));
    }

    Ok(ParsedSymbol {
        source,
        selector,
        params,
        anchor,
    })
}

/// 代入单个占位项：`ident`（直接查表）或 `ident±N`（整数算术）。
fn resolve_term(term: &str, state: &HashMap<String, String>, orig: &str) -> Result<String, CoreError> {
    let term = term.trim();
    if let Some(pos) = term.find(['+', '-']) {
        let (name, rest) = term.split_at(pos);
        let name = name.trim();
        let sign = &rest[..1];
        let offset: i64 = rest[1..]
            .trim()
            .parse()
            .map_err(|_| unresolved(orig, format!("算术占位偏移量非整数：{rest}")))?;
        let base: i64 = state
            .get(name)
            .ok_or_else(|| unresolved(orig, format!("状态缺少占位 {name}")))?
            .parse()
            .map_err(|_| unresolved(orig, format!("算术占位要求整数状态值：{name}")))?;
        let value = if sign == "+" { base + offset } else { base - offset };
        Ok(value.to_string())
    } else {
        state
            .get(term)
            .cloned()
            .ok_or_else(|| unresolved(orig, format!("状态缺少占位 {term}")))
    }
}

/// 代入 `{...}` 占位（支持逗号分隔多项，代入后以 `-` 连接）。
fn substitute(template: &str, state: &HashMap<String, String>, orig: &str) -> Result<String, CoreError> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after
            .find('}')
            .ok_or_else(|| unresolved(orig, "占位缺少闭合 }"))?;
        let body = &after[..end];
        let parts: Result<Vec<String>, CoreError> =
            body.split(',').map(|term| resolve_term(term, state, orig)).collect();
        out.push_str(&parts?.join("-"));
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// 由代入后的 source 基路径 + selector + params 组出项目相对路径。
///
/// - 有 selector：`base/{selector[-params]}.md`（如 `bible.style` → `bible/style.md`）。
/// - 无 selector 有 params：`{base}-{params}.md`（如 `prev_skeleton:{ch-1}` → `prev_skeleton-2.md`）。
/// - 无 selector 无 params：`base` 自带扩展名则原样（如 `workspace` → `CLAUDE.md`），否则补 `.md`。
fn build_path(base: &Path, selector: Option<&str>, params: Option<&str>) -> PathBuf {
    match selector {
        Some(sel) => {
            let stem = match params {
                Some(p) => format!("{sel}-{p}"),
                None => sel.to_string(),
            };
            base.join(format!("{stem}.md"))
        }
        None => match params {
            Some(p) => PathBuf::from(format!("{}-{p}.md", base.to_string_lossy())),
            None => {
                if base.extension().is_some() {
                    base.to_path_buf()
                } else {
                    PathBuf::from(format!("{}.md", base.to_string_lossy()))
                }
            }
        },
    }
}

/// 解析单条符号引用：切分 → 占位代入 → 经 `sources`（source→基路径）映射为
/// `SymbolRef`。`sources` 缺该 source 时回退为把 source 名当作目录/文件基路径。
pub fn resolve_symbol(
    raw: &str,
    state: &HashMap<String, String>,
    sources: &HashMap<String, PathBuf>,
) -> Result<SymbolRef, CoreError> {
    let parsed = parse_symbol(raw)?;

    let source = substitute(&parsed.source, state, raw)?;
    let selector = parsed
        .selector
        .as_deref()
        .map(|s| substitute(s, state, raw))
        .transpose()?;
    // params 在 parse 时已剥去外层 `{}`，代入前重新包回 `{...}` 交给占位扫描。
    let params = parsed
        .params
        .as_deref()
        .map(|p| substitute(&format!("{{{p}}}"), state, raw))
        .transpose()?;
    let anchor = parsed
        .anchor
        .as_deref()
        .map(|a| substitute(a, state, raw))
        .transpose()?;

    let base = sources
        .get(&source)
        .cloned()
        .unwrap_or_else(|| PathBuf::from(&source));
    let path = build_path(&base, selector.as_deref(), params.as_deref());

    Ok(SymbolRef {
        path,
        anchor,
        params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> HashMap<String, String> {
        HashMap::from([
            ("arc_id".to_string(), "arc7".to_string()),
            ("batch".to_string(), "b1".to_string()),
            ("ch".to_string(), "3".to_string()),
        ])
    }

    fn sources() -> HashMap<String, PathBuf> {
        HashMap::from([
            ("workspace".to_string(), PathBuf::from("CLAUDE.md")),
            ("spec".to_string(), PathBuf::from(".claude/agents")),
        ])
    }

    #[test]
    fn parses_source_selector_and_anchor() {
        let p = parse_symbol("bible.style#skeleton").unwrap();
        assert_eq!(p.source, "bible");
        assert_eq!(p.selector.as_deref(), Some("style"));
        assert_eq!(p.anchor.as_deref(), Some("skeleton"));
        assert!(p.params.is_none());
    }

    #[test]
    fn parses_selector_with_params() {
        let p = parse_symbol("plan.chapters:{batch}").unwrap();
        assert_eq!(p.source, "plan");
        assert_eq!(p.selector.as_deref(), Some("chapters"));
        assert_eq!(p.params.as_deref(), Some("batch"));
        assert!(p.anchor.is_none());
    }

    #[test]
    fn parses_source_only_with_params() {
        let p = parse_symbol("prev_skeleton:{ch-1}").unwrap();
        assert_eq!(p.source, "prev_skeleton");
        assert!(p.selector.is_none());
        assert_eq!(p.params.as_deref(), Some("ch-1"));
    }

    #[test]
    fn parses_pure_whole_file_source() {
        let p = parse_symbol("workspace").unwrap();
        assert_eq!(p.source, "workspace");
        assert!(p.selector.is_none());
        assert!(p.params.is_none());
        assert!(p.anchor.is_none());
    }

    #[test]
    fn substitutes_simple_and_arithmetic_placeholders() {
        let st = state();
        assert_eq!(substitute("{batch}", &st, "x").unwrap(), "b1");
        assert_eq!(substitute("{arc_id}", &st, "x").unwrap(), "arc7");
        // {ch-1}：当前章 3 → 前一章 2。
        assert_eq!(substitute("{ch-1}", &st, "x").unwrap(), "2");
        assert_eq!(substitute("{ch+2}", &st, "x").unwrap(), "5");
        // 多项逗号 → 以 - 连接。
        assert_eq!(substitute("{arc_id,batch}", &st, "x").unwrap(), "arc7-b1");
    }

    #[test]
    fn resolves_known_source_whole_file() {
        let r = resolve_symbol("workspace", &state(), &sources()).unwrap();
        assert_eq!(r.path, PathBuf::from("CLAUDE.md"));
        assert!(r.anchor.is_none());
    }

    #[test]
    fn resolves_known_source_with_selector_and_anchor() {
        let r = resolve_symbol("spec.brief-agent#inputs", &state(), &sources()).unwrap();
        assert_eq!(r.path, PathBuf::from(".claude/agents/brief-agent.md"));
        assert_eq!(r.anchor.as_deref(), Some("inputs"));
    }

    #[test]
    fn resolves_unknown_source_falls_back_to_dir() {
        let r = resolve_symbol("bible.style#skeleton", &state(), &sources()).unwrap();
        assert_eq!(r.path, PathBuf::from("bible/style.md"));
        assert_eq!(r.anchor.as_deref(), Some("skeleton"));
    }

    #[test]
    fn resolves_params_into_path_stem() {
        let r = resolve_symbol("plan.chapters:{batch}", &state(), &sources()).unwrap();
        assert_eq!(r.path, PathBuf::from("plan/chapters-b1.md"));
        assert_eq!(r.params.as_deref(), Some("b1"));

        let r2 = resolve_symbol("prev_skeleton:{ch-1}", &state(), &sources()).unwrap();
        assert_eq!(r2.path, PathBuf::from("prev_skeleton-2.md"));
    }

    #[test]
    fn missing_placeholder_state_is_unresolved() {
        let err = resolve_symbol("plan.chapters:{missing}", &state(), &sources()).unwrap_err();
        assert!(matches!(err, CoreError::SymbolUnresolved { .. }));
    }

    #[test]
    fn malformed_params_is_unresolved() {
        let err = parse_symbol("plan.chapters:batch").unwrap_err();
        assert!(matches!(err, CoreError::SymbolUnresolved { .. }));
    }

    #[test]
    fn empty_anchor_is_unresolved() {
        let err = parse_symbol("bible.style#").unwrap_err();
        assert!(matches!(err, CoreError::SymbolUnresolved { .. }));
    }
}
