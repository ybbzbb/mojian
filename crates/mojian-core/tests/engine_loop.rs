//! TASK-005 集成测试：在隔离 `MOJIAN_HOME` + 临时中央 DB 中登记项目、种子 volume /
//! chapter 行，真实调用 `apply_generation` / `apply_decision` 并回读 DB 断言状态机推进
//! （brief 关卡 → CONFIRMED 推进 vision_drafting；VOID 章节最小语义 void → planned）。

use std::path::PathBuf;

use mojian_core::{
    apply_decision, apply_generation, open_central_db, register_project, InputSlice, Verdict,
    BRIEF_GATE,
};
use rusqlite::{params, Connection};

/// 将 `MOJIAN_HOME` 指向隔离临时目录，避免污染真实数据目录（QA 命令口径一致）。
fn isolate_home() -> PathBuf {
    let base = std::env::temp_dir().join(format!("mojian-engine-test-{}", std::process::id()));
    std::env::set_var("MOJIAN_HOME", &base);
    base
}

/// 唯一临时 DB 路径，按纳秒 + pid 分区，避免并行测试互相踩。
fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mojian-engine-{}-{}-{}",
        tag,
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("central.db")
}

/// 打开临时中央 DB 并登记一个新项目（sop_phase 初值 style_sampling）。
fn setup(tag: &str) -> (Connection, String) {
    let _home = isolate_home();
    let mut conn = open_central_db(temp_db_path(tag)).expect("open central db");
    let project_id = register_project(&mut conn, tag, &PathBuf::from("/tmp").join(tag))
        .expect("register project");
    (conn, project_id)
}

fn set_phase(conn: &Connection, project_id: &str, phase: &str) {
    conn.execute(
        "UPDATE project_state SET sop_phase = ?1 WHERE project_id = ?2",
        params![phase, project_id],
    )
    .unwrap();
}

fn phase(conn: &Connection, project_id: &str) -> String {
    conn.query_row(
        "SELECT sop_phase FROM project_state WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )
    .unwrap()
}

fn cursors(conn: &Connection, project_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT cursors FROM project_state WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn generation_sets_brief_gate_then_confirm_advances() {
    let (conn, pid) = setup("confirm");
    set_phase(&conn, &pid, "brief_drafting");

    let inputs = vec![
        InputSlice {
            path: "bible/style.md".to_string(),
            anchor: Some("skeleton".to_string()),
            content_hash: "hash-style".to_string(),
        },
        InputSlice {
            path: "creative/creative-brief.md".to_string(),
            anchor: None,
            content_hash: "hash-brief".to_string(),
        },
    ];

    apply_generation(&conn, &pid, BRIEF_GATE, &inputs).unwrap();

    // 回读 cursors：应含 pending_gate == "brief"。
    let raw = cursors(&conn, &pid).expect("cursors should be set");
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["pending_gate"], "brief");

    // artifact_ref 应为每个输入切片各写一行（content_hash 与 generation.jsonl 同源）。
    let artifact_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM artifact_ref WHERE project_id = ?1 AND kind = 'input'",
            params![pid],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(artifact_count, 2);

    // CONFIRMED：清关卡 + 推进 brief_drafting → vision_drafting。
    apply_decision(&conn, &pid, BRIEF_GATE, Verdict::Confirmed, None).unwrap();

    let raw = cursors(&conn, &pid);
    let cleared = raw
        .map(|text| {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            v.get("pending_gate").is_none()
        })
        .unwrap_or(true);
    assert!(cleared, "CONFIRMED 后 pending_gate 应被清除");
    assert_eq!(phase(&conn, &pid), "vision_drafting");
}

#[test]
fn revise_at_brief_clears_gate_and_stays_at_brief_drafting() {
    let (conn, pid) = setup("revise");
    set_phase(&conn, &pid, "brief_drafting");

    apply_generation(&conn, &pid, BRIEF_GATE, &[]).unwrap();
    apply_decision(&conn, &pid, BRIEF_GATE, Verdict::Revise, None).unwrap();

    let raw = cursors(&conn, &pid);
    let cleared = raw
        .map(|text| {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            v.get("pending_gate").is_none()
        })
        .unwrap_or(true);
    assert!(cleared, "REVISE 后 pending_gate 应被清除");
    assert_eq!(phase(&conn, &pid), "brief_drafting", "REVISE 回退细粒度状态");
}

#[test]
fn void_chapter_records_and_resets_to_planned() {
    let (conn, pid) = setup("void");

    // 种子 volume + 一个 status == "void" 的 chapter 行。
    let now = "2026-07-08T00:00:00Z";
    conn.execute(
        "INSERT INTO volume (id, project_id, seq, arc_phase, updated_at) \
         VALUES ('ARC-1', ?1, 1, 'writing', ?2)",
        params![pid, now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO chapter (id, project_id, volume_id, seq, status, updated_at) \
         VALUES ('CH-7', ?1, 'ARC-1', 7, 'void', ?2)",
        params![pid, now],
    )
    .unwrap();

    apply_decision(&conn, &pid, "skeleton_review", Verdict::Void, Some("CH-7")).unwrap();

    let void_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM void_record WHERE project_id = ?1 AND chapter_id = 'CH-7'",
            params![pid],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(void_rows, 1, "VOID 应新增一条 void_record");

    let status: String = conn
        .query_row(
            "SELECT status FROM chapter WHERE id = 'CH-7'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "planned", "裁决③：void → planned，不级联");
}
