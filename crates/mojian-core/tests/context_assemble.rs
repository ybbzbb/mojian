//! TASK-004 集成测试：在隔离 `MOJIAN_HOME` + 临时项目目录中部署占位 SPEC、种子最小 DB
//! 行，真实调用 `assemble_bundle` 解析磁盘上的 `brief-agent.manifest.toml`，验证五字段
//! 装配 + write_scope 推导 + 切片 content_hash + decision.jsonl 人类评论回喂（REQ-003/011）。

use std::path::{Path, PathBuf};

use mojian_core::context::slice_ref;
use mojian_core::{
    append_decision, assemble_bundle, deploy_spec, embedded_spec, ensure_master, open_central_db,
    register_project, DecisionEvent,
};

/// 将 `MOJIAN_HOME` 指向隔离临时目录；所有测试指向同一 base，靠唯一 `project_id` 分区。
fn isolate_home() -> PathBuf {
    let base = std::env::temp_dir().join(format!("mojian-context-test-{}", std::process::id()));
    std::env::set_var("MOJIAN_HOME", &base);
    base
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-context-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 部署占位 SPEC 到一个临时项目目录，返回项目根路径。
fn deploy_project(tag: &str) -> PathBuf {
    let master = unique_dir(&format!("{tag}-master")).join("spec");
    ensure_master(embedded_spec(), &master).expect("bootstrap master");
    let project = unique_dir(&format!("{tag}-project"));
    deploy_spec(&master, &project).expect("deploy spec");
    project
}

fn manifest_path(project: &Path) -> PathBuf {
    project.join(".claude/agents/brief-agent.manifest.toml")
}

#[test]
fn assemble_bundle_end_to_end_with_comment_feedback() {
    let _home = isolate_home();
    let project = deploy_project("e2e");

    // 部署产物必然存在（占位 manifest inputs 指向它们）。
    assert!(project.join("CLAUDE.md").is_file());
    assert!(project.join(".claude/agents/brief-agent.md").is_file());
    assert!(manifest_path(&project).is_file());

    // 种子最小 DB：登记项目得到 project_id + 初始 SOP 状态行。
    let mut conn = open_central_db(project.join("central.db")).unwrap();
    let project_id = register_project(&mut conn, "brief-e2e", &project).unwrap();

    // 预先向 decision.jsonl 写入一条 gate == "brief" 且带 comment 的记录（REQ-011）。
    let comment = "把题材收紧到都市悬疑，弱化群像";
    append_decision(
        &project_id,
        &DecisionEvent {
            gate: "brief".to_string(),
            verdict: "REVISE".to_string(),
            target: None,
            comment: Some(comment.to_string()),
            ts: "2026-07-08T00:00:00Z".to_string(),
        },
    )
    .unwrap();

    let bundle = assemble_bundle(&conn, &project_id, &project, &manifest_path(&project)).unwrap();

    // agent 指向部署的 brief-agent。
    assert_eq!(bundle.agent, ".claude/agents/brief-agent.md");

    // write_scope 由 manifest `write:` 推导：非空、与白名单一致。
    assert_eq!(
        bundle.write_scope,
        vec![PathBuf::from("creative/creative-brief.md")]
    );
    assert!(!bundle.write_scope.is_empty());

    // spec_slice 为 agent 提示词整文件（非空）。
    assert!(bundle.spec_slice.contains("brief-agent"));

    // inputs 含被切片文件的 content_hash（整文件 CLAUDE.md + 段级 brief-agent#inputs）。
    let whole = slice_ref(&project, Path::new("CLAUDE.md"), None).unwrap();
    let section = slice_ref(
        &project,
        Path::new(".claude/agents/brief-agent.md"),
        Some("inputs"),
    )
    .unwrap();
    assert!(
        bundle.inputs.contains(&whole.content_hash),
        "inputs 应含整文件切片 content_hash"
    );
    assert!(
        bundle.inputs.contains(&section.content_hash),
        "inputs 应含段级切片 content_hash"
    );

    // 段级切片只取 `## inputs` 段，不越界到 `## output`。
    assert!(section.content.contains("题材与定位"));
    assert!(!section.content.contains("## output"));

    // decision.jsonl 的人类评论被回喂进 inputs（REQ-011）。
    assert!(
        bundle.inputs.contains(comment),
        "inputs 应回喂 brief 关卡的人类评论"
    );
}

#[test]
fn assemble_bundle_without_comments_has_no_feedback_block() {
    let _home = isolate_home();
    let project = deploy_project("nocomment");

    let mut conn = open_central_db(project.join("central.db")).unwrap();
    let project_id = register_project(&mut conn, "brief-nocomment", &project).unwrap();

    let bundle = assemble_bundle(&conn, &project_id, &project, &manifest_path(&project)).unwrap();

    // 无 decision 记录时，inputs 仍含切片但不含回喂评论块。
    assert!(bundle.inputs.contains("## input:"));
    assert!(!bundle.inputs.contains("human comments"));
}
