use std::fs::{self, OpenOptions};
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::paths::logs_dir;

const GENERATION_LOG_FILE: &str = "generation.jsonl";
const DECISION_LOG_FILE: &str = "decision.jsonl";

/// generation.jsonl 中单条输入切片：切片来源（`path` 可整文件、`anchor` 标注段级
/// `#anchor`）及其内容 hash（与 `artifact_ref.content_hash` 同源，供过期检测 / token 对账）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSlice {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    pub content_hash: String,
}

/// 一次 SDK 生成事件（写入 generation.jsonl，一行一条，只增不改）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationEvent {
    pub step: String,
    pub agent: String,
    pub spec_path: String,
    pub spec_hash: String,
    pub inputs: Vec<InputSlice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_out: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    pub ts: String,
}

/// 人在关卡的决定事件（写入 decision.jsonl，一行一条，只增不改）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    pub gate: String,
    pub verdict: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub ts: String,
}

fn append_jsonl<T: Serialize>(
    project_id: &str,
    file_name: &str,
    event: &T,
) -> Result<(), CoreError> {
    let dir = logs_dir()?.join(project_id);
    fs::create_dir_all(&dir).map_err(|source| CoreError::Io {
        path: dir.clone(),
        source,
    })?;

    let path = dir.join(file_name);
    let line = serde_json::to_string(event)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| CoreError::Io {
            path: path.clone(),
            source,
        })?;

    writeln!(file, "{line}").map_err(|source| CoreError::Io { path, source })
}

/// 追加写一条生成事件到 `<data_dir>/logs/{project_id}/generation.jsonl`。
pub fn append_generation(project_id: &str, event: &GenerationEvent) -> Result<(), CoreError> {
    append_jsonl(project_id, GENERATION_LOG_FILE, event)
}

/// 追加写一条决定事件到 `<data_dir>/logs/{project_id}/decision.jsonl`。
pub fn append_decision(project_id: &str, event: &DecisionEvent) -> Result<(), CoreError> {
    append_jsonl(project_id, DECISION_LOG_FILE, event)
}

/// 匹配单条决定事件：gate 相同、comment 非空，且 target 命中（查询 target 为 `None`
/// 视为全局取全部；事件 target 为空视为全局评论、对任意查询 target 均命中）。
fn pick_comment(event: &DecisionEvent, gate: &str, target: Option<&str>) -> Option<String> {
    if event.gate != gate {
        return None;
    }
    let comment = event.comment.as_deref().filter(|c| !c.is_empty())?;
    let hit = match target {
        None => true,
        Some(t) => event.target.as_deref().map_or(true, |et| et == t),
    };
    hit.then(|| comment.to_string())
}

/// 回读 `decision.jsonl` 中匹配 `gate`（且 `target` 命中）且评论非空的评论，按写入顺序
/// 返回（REQ-011 装配回喂来源）。文件不存在返回空 `Vec`。
pub fn read_decision_comments(
    project_id: &str,
    gate: &str,
    target: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    let path = logs_dir()?.join(project_id).join(DECISION_LOG_FILE);

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(CoreError::Io { path, source }),
    };

    let mut comments = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let event: DecisionEvent = serde_json::from_str(line)?;
        if let Some(comment) = pick_comment(&event, gate, target) {
            comments.push(comment);
        }
    }
    Ok(comments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_generation() -> GenerationEvent {
        GenerationEvent {
            step: "brief_drafting".to_string(),
            agent: ".claude/agents/brief-agent.md".to_string(),
            spec_path: "spec.toml".to_string(),
            spec_hash: "abc123".to_string(),
            inputs: vec![
                InputSlice {
                    path: "bible/style.md".to_string(),
                    anchor: Some("skeleton".to_string()),
                    content_hash: "hash-a".to_string(),
                },
                InputSlice {
                    path: "creative/creative-brief.md".to_string(),
                    anchor: None,
                    content_hash: "hash-b".to_string(),
                },
            ],
            token_in: Some(1200),
            token_out: Some(800),
            cost: Some(0.0123),
            ts: "2026-07-08T00:00:00Z".to_string(),
        }
    }

    fn decision(gate: &str, target: Option<&str>, comment: Option<&str>) -> DecisionEvent {
        DecisionEvent {
            gate: gate.to_string(),
            verdict: "REVISE".to_string(),
            target: target.map(str::to_string),
            comment: comment.map(str::to_string),
            ts: "2026-07-08T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn generation_serializes_to_single_line_json() {
        let line = serde_json::to_string(&sample_generation()).unwrap();
        assert!(!line.contains('\n'), "序列化结果必须为单行");

        let value: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["step"], "brief_drafting");
        assert_eq!(value["agent"], ".claude/agents/brief-agent.md");
        assert_eq!(value["spec_path"], "spec.toml");
        assert_eq!(value["spec_hash"], "abc123");
        assert_eq!(value["token_in"], 1200);
        assert_eq!(value["token_out"], 800);
        assert_eq!(value["inputs"][0]["path"], "bible/style.md");
        assert_eq!(value["inputs"][0]["anchor"], "skeleton");
        assert_eq!(value["inputs"][0]["content_hash"], "hash-a");
        // 整文件切片无 anchor：Option::None 被跳过，不落 key。
        assert!(value["inputs"][1].get("anchor").is_none());
        assert_eq!(value["inputs"][1]["content_hash"], "hash-b");
    }

    #[test]
    fn decision_round_trips_and_omits_empty_optionals() {
        let event = decision("brief", None, None);
        let line = serde_json::to_string(&event).unwrap();
        assert!(!line.contains('\n'));

        let value: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["gate"], "brief");
        assert_eq!(value["verdict"], "REVISE");
        assert!(value.get("target").is_none());
        assert!(value.get("comment").is_none());

        let back: DecisionEvent = serde_json::from_str(&line).unwrap();
        assert_eq!(back.gate, "brief");
        assert!(back.target.is_none());
        assert!(back.comment.is_none());
    }

    #[test]
    fn pick_comment_filters_by_gate() {
        let hit = decision("brief", None, Some("收紧定位"));
        let miss = decision("vision", None, Some("换金手指"));
        assert_eq!(pick_comment(&hit, "brief", None).as_deref(), Some("收紧定位"));
        assert_eq!(pick_comment(&miss, "brief", None), None);
    }

    #[test]
    fn pick_comment_skips_empty_or_missing_comment() {
        assert_eq!(pick_comment(&decision("brief", None, None), "brief", None), None);
        assert_eq!(
            pick_comment(&decision("brief", None, Some("")), "brief", None),
            None
        );
    }

    #[test]
    fn pick_comment_target_boundaries() {
        // 查询 target 为 None：命中任意事件 target。
        let ev = decision("skeleton_review", Some("CH-003"), Some("补钩子"));
        assert_eq!(
            pick_comment(&ev, "skeleton_review", None).as_deref(),
            Some("补钩子")
        );

        // 查询指定 target：完全匹配命中。
        assert_eq!(
            pick_comment(&ev, "skeleton_review", Some("CH-003")).as_deref(),
            Some("补钩子")
        );

        // 查询指定 target：不匹配的事件 target 被过滤。
        assert_eq!(pick_comment(&ev, "skeleton_review", Some("CH-004")), None);

        // 事件 target 为空（全局评论）：对任意查询 target 均命中。
        let global = decision("skeleton_review", None, Some("全局提醒"));
        assert_eq!(
            pick_comment(&global, "skeleton_review", Some("CH-004")).as_deref(),
            Some("全局提醒")
        );
    }
}
