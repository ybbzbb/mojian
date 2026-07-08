//! 切片装配器：读 manifest → 解析符号 → 切片 → 回喂 decision 评论 → 组五字段 `Bundle`。
//!
//! `assemble_bundle` 是 IMPL-3 的核心入口（REQ-003 / REQ-011），把「当前 DB 状态 + 输入
//! 契约 manifest + 磁盘 SSOT + 人类关卡评论」装配成一次 SDK 调用的最小上下文。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::CoreError;
use crate::log::read_decision_comments;
use crate::project::load_project_state;
use crate::sdk::Bundle;

use super::manifest::{read_input_manifest, write_scope};
use super::slice::{slice_ref, Slice};
use super::symbol::resolve_symbol;

/// 为符号解析构造默认 source→项目相对基路径映射。
///
/// 占位步骤所需两项：`workspace` → 部署的 `CLAUDE.md`（整文件）、`spec` → `.claude/agents`
/// 目录（`spec.<agent>#<anchor>` → 段级）。未登记的 source 由解析器回退为把名字当基路径，
/// 兼容 `bible.style#skeleton` 等真实 SSOT 引用（后续迭代接线时扩展本表）。
fn default_sources() -> HashMap<String, PathBuf> {
    HashMap::from([
        ("workspace".to_string(), PathBuf::from("CLAUDE.md")),
        ("spec".to_string(), PathBuf::from(".claude/agents")),
    ])
}

/// 按当前 DB 状态构造符号占位代入表（`{phase}` 等）。占位步骤仅需 `phase`；后续接入
/// 章节/批次推进时在此补 `{arc_id}` / `{batch}` / `{ch}` 等键。
fn state_map(conn: &Connection, project_id: &str) -> Result<HashMap<String, String>, CoreError> {
    let phase = load_project_state(conn, project_id)?;
    Ok(HashMap::from([(
        "phase".to_string(),
        phase.as_db_str().to_string(),
    )]))
}

/// 把切片与回喂评论渲染成 `Bundle.inputs` 文本：每片带路径/锚点/内容 hash + 内容，
/// 末尾附本关卡的人类评论（REQ-011 回喂）。
fn render_inputs(slices: &[Slice], gate: Option<&str>, comments: &[String]) -> String {
    let mut out = String::new();
    for slice in slices {
        let anchor = slice
            .anchor
            .as_deref()
            .map(|a| format!("#{a}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "## input: {}{} (hash: {})\n{}\n\n",
            slice.rel_path, anchor, slice.content_hash, slice.content
        ));
    }
    if !comments.is_empty() {
        out.push_str(&format!("## human comments (gate: {})\n", gate.unwrap_or("-")));
        for comment in comments {
            out.push_str(&format!("- {comment}\n"));
        }
    }
    out
}

/// 装配一次 SDK 调用的五字段 `Bundle`。
///
/// - `conn` / `project_id`：读当前 SOP 状态（符号占位代入）+ 回喂 decision.jsonl 评论。
/// - `project_dir`：磁盘上项目根，切片相对其解析。
/// - `manifest_path`：TOML sidecar 输入契约路径。
pub fn assemble_bundle(
    conn: &Connection,
    project_id: &str,
    project_dir: &Path,
    manifest_path: &Path,
) -> Result<Bundle, CoreError> {
    let manifest = read_input_manifest(manifest_path)?;

    let state = state_map(conn, project_id)?;
    let sources = default_sources();

    let mut slices: Vec<Slice> = Vec::with_capacity(manifest.inputs.len());
    for symbol in &manifest.inputs {
        let symbol_ref = resolve_symbol(symbol, &state, &sources)?;
        slices.push(slice_ref(project_dir, &symbol_ref.path, symbol_ref.anchor.as_deref())?);
    }

    let spec_slice = {
        let abs = project_dir.join(&manifest.agent);
        std::fs::read_to_string(&abs).map_err(|source| CoreError::Io { path: abs, source })?
    };

    let comments = match manifest.gate.as_deref() {
        Some(gate) => read_decision_comments(project_id, gate, None)?,
        None => Vec::new(),
    };

    let inputs = render_inputs(&slices, manifest.gate.as_deref(), &comments);
    let write_scope = write_scope(&manifest);

    Ok(Bundle {
        agent: manifest.agent,
        spec_slice,
        inputs,
        write_scope,
        output_contract: manifest.output_contract,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_inputs_embeds_hash_and_comments() {
        let slices = vec![Slice {
            rel_path: "CLAUDE.md".to_string(),
            anchor: None,
            content: "工作区说明".to_string(),
            content_hash: "deadbeef".to_string(),
        }];
        let out = render_inputs(&slices, Some("brief"), &["收紧题材定位".to_string()]);
        assert!(out.contains("CLAUDE.md"));
        assert!(out.contains("deadbeef"));
        assert!(out.contains("工作区说明"));
        assert!(out.contains("gate: brief"));
        assert!(out.contains("收紧题材定位"));
    }

    #[test]
    fn render_inputs_omits_comment_block_when_empty() {
        let slices = vec![Slice {
            rel_path: "a.md".to_string(),
            anchor: Some("inputs".to_string()),
            content: "段落".to_string(),
            content_hash: "hash".to_string(),
        }];
        let out = render_inputs(&slices, Some("brief"), &[]);
        assert!(out.contains("a.md#inputs"));
        assert!(!out.contains("human comments"));
    }

    #[test]
    fn default_sources_cover_placeholder_step() {
        let sources = default_sources();
        assert_eq!(sources.get("workspace"), Some(&PathBuf::from("CLAUDE.md")));
        assert_eq!(sources.get("spec"), Some(&PathBuf::from(".claude/agents")));
    }
}
