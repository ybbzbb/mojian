//! 输入契约 manifest：部署 SPEC 内的 TOML sidecar（选型 3-A）。
//!
//! 每个可跑步骤在 `.claude/agents/<agent>.manifest.toml` 声明 `agent` / `inputs`
//! （符号引用数组）/ `write`（写白名单）/ `output_contract`，可选 `gate`（该步骤对应
//! 的人工关卡，用于装配时回喂 decision.jsonl 评论）。Rust 装配器读契约，`claude`
//! 原生读同目录 `<agent>.md` 提示词，各取所需。

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::CoreError;

/// TOML sidecar 输入契约模型。非声明字段以 serde `default` 容错为空。
#[derive(Debug, Clone, Deserialize)]
pub struct InputManifest {
    /// 本步引用的部署 agent 相对路径（如 `.claude/agents/brief-agent.md`）。
    pub agent: String,
    /// 本步对应的人工关卡名（如 `brief`）；装配时据此回喂 decision.jsonl 评论。
    #[serde(default)]
    pub gate: Option<String>,
    /// 符号引用数组（`<source>.<selector>[:{params}][#anchor]`），装配器逐一解析切片。
    #[serde(default)]
    pub inputs: Vec<String>,
    /// 写白名单，推导为 `Bundle.write_scope`（沙箱写沙盒）。
    #[serde(default)]
    pub write: Vec<String>,
    /// 期望产出与 done 信号形状。
    #[serde(default)]
    pub output_contract: String,
}

/// 读取并反序列化 TOML sidecar 输入契约。文件缺失返回 `CoreError::Io`；
/// TOML 非法或缺必填字段返回 `CoreError::ManifestInvalid`。
pub fn read_input_manifest(path: &Path) -> Result<InputManifest, CoreError> {
    let text = std::fs::read_to_string(path).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|error| CoreError::ManifestInvalid {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

/// 把 manifest `write` 白名单条目推导为 `Bundle.write_scope`（逐项作为相对路径沙箱条目）。
pub fn write_scope(manifest: &InputManifest) -> Vec<PathBuf> {
    manifest.write.iter().map(PathBuf::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mojian-manifest-test-{}-{}",
            std::process::id(),
            name
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("manifest.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn parses_full_manifest() {
        let path = write_temp(
            "full",
            r#"
agent = ".claude/agents/brief-agent.md"
gate = "brief"
inputs = ["workspace", "spec.brief-agent#inputs"]
write = ["creative/creative-brief.md"]
output_contract = "写入 creative-brief.md"
"#,
        );
        let manifest = read_input_manifest(&path).unwrap();
        assert_eq!(manifest.agent, ".claude/agents/brief-agent.md");
        assert_eq!(manifest.gate.as_deref(), Some("brief"));
        assert_eq!(manifest.inputs.len(), 2);
        assert_eq!(write_scope(&manifest), vec![PathBuf::from("creative/creative-brief.md")]);
        assert_eq!(manifest.output_contract, "写入 creative-brief.md");
    }

    #[test]
    fn missing_optional_fields_default_to_empty() {
        let path = write_temp("minimal", "agent = \".claude/agents/x.md\"\n");
        let manifest = read_input_manifest(&path).unwrap();
        assert!(manifest.gate.is_none());
        assert!(manifest.inputs.is_empty());
        assert!(write_scope(&manifest).is_empty());
        assert!(manifest.output_contract.is_empty());
    }

    #[test]
    fn invalid_toml_returns_manifest_invalid() {
        let path = write_temp("broken", "agent = = = broken");
        let err = read_input_manifest(&path).unwrap_err();
        assert!(matches!(err, CoreError::ManifestInvalid { .. }));
    }

    #[test]
    fn missing_required_agent_returns_manifest_invalid() {
        let path = write_temp("noagent", "gate = \"brief\"\n");
        let err = read_input_manifest(&path).unwrap_err();
        assert!(matches!(err, CoreError::ManifestInvalid { .. }));
    }
}
