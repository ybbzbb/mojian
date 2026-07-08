use mojian_core::{
    append_decision, append_generation, read_decision_comments, DecisionEvent, GenerationEvent,
    InputSlice,
};

/// 将 `MOJIAN_HOME` 指向隔离临时目录，避免污染真实数据目录。所有测试指向同一 base，
/// 靠各自唯一的 `project_id` 分区隔离；set 同一值对并行执行安全。
fn isolate_home() -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!("mojian-log-test-{}", std::process::id()));
    std::env::set_var("MOJIAN_HOME", &base);
    base
}

fn generation(step: &str, hash: &str) -> GenerationEvent {
    GenerationEvent {
        step: step.to_string(),
        agent: ".claude/agents/brief-agent.md".to_string(),
        spec_path: "spec.toml".to_string(),
        spec_hash: "spec-hash".to_string(),
        inputs: vec![InputSlice {
            path: "bible/style.md".to_string(),
            anchor: Some("skeleton".to_string()),
            content_hash: hash.to_string(),
        }],
        token_in: Some(100),
        token_out: Some(50),
        cost: Some(0.001),
        ts: "2026-07-08T00:00:00Z".to_string(),
    }
}

fn decision(gate: &str, comment: &str) -> DecisionEvent {
    DecisionEvent {
        gate: gate.to_string(),
        verdict: "REVISE".to_string(),
        target: None,
        comment: Some(comment.to_string()),
        ts: "2026-07-08T00:00:00Z".to_string(),
    }
}

#[test]
fn append_generation_is_append_only_two_lines() {
    let base = isolate_home();
    let project_id = "proj-generation-append";

    append_generation(project_id, &generation("brief_drafting", "hash-1")).unwrap();
    let first_line = {
        let path = base
            .join("logs")
            .join(project_id)
            .join("generation.jsonl");
        std::fs::read_to_string(&path).unwrap()
    };

    append_generation(project_id, &generation("vision_drafting", "hash-2")).unwrap();

    let path = base
        .join("logs")
        .join(project_id)
        .join("generation.jsonl");
    assert!(path.exists(), "generation.jsonl 应已落盘");

    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "连续两次 append 后应为两行");

    // 只增不改：第二次写入后，第一行内容不变。
    assert!(
        content.starts_with(first_line.trim_end()),
        "第一行内容不得被第二次写入改动"
    );

    // 每行为合法 JSON 且字段还原正确。
    let e0: GenerationEvent = serde_json::from_str(lines[0]).unwrap();
    let e1: GenerationEvent = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(e0.step, "brief_drafting");
    assert_eq!(e0.inputs[0].content_hash, "hash-1");
    assert_eq!(e1.step, "vision_drafting");
    assert_eq!(e1.inputs[0].content_hash, "hash-2");
}

#[test]
fn read_decision_comments_filters_by_gate() {
    isolate_home();
    let project_id = "proj-decision-filter";

    append_decision(project_id, &decision("brief", "收紧题材定位")).unwrap();
    append_decision(project_id, &decision("vision", "更换金手指")).unwrap();

    let comments = read_decision_comments(project_id, "brief", None).unwrap();
    assert_eq!(comments, vec!["收紧题材定位".to_string()]);
}

#[test]
fn read_decision_comments_missing_file_returns_empty() {
    isolate_home();
    let comments = read_decision_comments("proj-never-written", "brief", None).unwrap();
    assert!(comments.is_empty());
}
