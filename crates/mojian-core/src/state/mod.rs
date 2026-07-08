//! 运行时 DB 行读写：状态机推进的落库面。
//!
//! 承载 `project_state.sop_phase` 推进、SOP① 顺序关卡标记（借 `project_state.cursors`
//! JSON 的 `pending_gate` 字段，免 schema 迁移）、`chapter` 状态读写、`void_record` 插入、
//! `artifact_ref` upsert。全部走 rusqlite，`SCHEMA_VERSION` 保持 1，不加列。

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::domain::ChapterState;
use crate::error::CoreError;

/// `cursors` JSON 中承载 SOP① 顺序关卡 pending 状态的字段名。
const PENDING_GATE_KEY: &str = "pending_gate";

/// 当前 UTC 时刻的 RFC3339 文本，写入 TEXT 时间列。
fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 formatting of the current UTC time is infallible")
}

/// 从 `chapter` 行读回的最小运行时视图（本迭代状态机所需列）。
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: String,
    pub project_id: String,
    pub volume_id: String,
    pub batch_id: Option<String>,
    pub seq: i64,
    pub status: ChapterState,
}

/// 推进 `project_state.sop_phase` 到目标 phase。
pub fn advance_sop_phase(
    conn: &Connection,
    project_id: &str,
    phase: crate::domain::SopPhase,
) -> Result<(), CoreError> {
    conn.execute(
        "UPDATE project_state SET sop_phase = ?1, updated_at = ?2 WHERE project_id = ?3",
        params![phase.as_db_str(), now_rfc3339(), project_id],
    )?;
    Ok(())
}

/// 读回 `project_state.cursors` JSON 为对象 map；列为 NULL / 空 / 非对象时视为空 map。
fn read_cursors(conn: &Connection, project_id: &str) -> Result<Map<String, Value>, CoreError> {
    let raw: Option<String> = conn.query_row(
        "SELECT cursors FROM project_state WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )?;

    match raw {
        Some(text) if !text.trim().is_empty() => match serde_json::from_str::<Value>(&text)? {
            Value::Object(map) => Ok(map),
            _ => Ok(Map::new()),
        },
        _ => Ok(Map::new()),
    }
}

/// 写回 `project_state.cursors`：空 map 落 NULL，否则序列化为 JSON 文本。
fn write_cursors(
    conn: &Connection,
    project_id: &str,
    map: Map<String, Value>,
) -> Result<(), CoreError> {
    let text = if map.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&Value::Object(map))?)
    };
    conn.execute(
        "UPDATE project_state SET cursors = ?1, updated_at = ?2 WHERE project_id = ?3",
        params![text, now_rfc3339(), project_id],
    )?;
    Ok(())
}

/// 置 SOP① 顺序关卡标记：在 `cursors` JSON 写入 `pending_gate = <gate>`。
pub fn set_gate(conn: &Connection, project_id: &str, gate: &str) -> Result<(), CoreError> {
    let mut map = read_cursors(conn, project_id)?;
    map.insert(PENDING_GATE_KEY.to_string(), Value::String(gate.to_string()));
    write_cursors(conn, project_id, map)
}

/// 清除 SOP① 顺序关卡标记：从 `cursors` JSON 移除 `pending_gate`。
pub fn clear_gate(conn: &Connection, project_id: &str) -> Result<(), CoreError> {
    let mut map = read_cursors(conn, project_id)?;
    map.remove(PENDING_GATE_KEY);
    write_cursors(conn, project_id, map)
}

/// 读回当前 SOP① 顺序关卡标记（`cursors.pending_gate`），无标记返回 `None`。
pub fn read_pending_gate(conn: &Connection, project_id: &str) -> Result<Option<String>, CoreError> {
    let map = read_cursors(conn, project_id)?;
    Ok(map
        .get(PENDING_GATE_KEY)
        .and_then(Value::as_str)
        .map(str::to_string))
}

/// 按 `id` 读回一个 `chapter` 行的最小运行时视图。无此章节返回
/// `CoreError::Db(QueryReturnedNoRows)`；`status` 文本非法返回 `UnknownDomainValue`。
pub fn load_chapter(conn: &Connection, chapter_id: &str) -> Result<Chapter, CoreError> {
    let (id, project_id, volume_id, batch_id, seq, status_text): (
        String,
        String,
        String,
        Option<String>,
        i64,
        String,
    ) = conn.query_row(
        "SELECT id, project_id, volume_id, batch_id, seq, status FROM chapter WHERE id = ?1",
        params![chapter_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;

    Ok(Chapter {
        id,
        project_id,
        volume_id,
        batch_id,
        seq,
        status: ChapterState::try_from(status_text.as_str())?,
    })
}

/// 更新 `chapter.status` 到目标状态。
pub fn update_chapter_status(
    conn: &Connection,
    chapter_id: &str,
    status: ChapterState,
) -> Result<(), CoreError> {
    conn.execute(
        "UPDATE chapter SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_db_str(), now_rfc3339(), chapter_id],
    )?;
    Ok(())
}

/// 插入一条 `void_record`（作废记录）。裁决③ 最小语义：`reason` / `affected_scope`
/// 可空，本迭代不做圣经级联，`affected_scope` 留空。
pub fn insert_void_record(
    conn: &Connection,
    project_id: &str,
    chapter_id: &str,
    reason: Option<&str>,
    affected_scope: Option<&str>,
) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO void_record (project_id, chapter_id, reason, affected_scope, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![project_id, chapter_id, reason, affected_scope, now_rfc3339()],
    )?;
    Ok(())
}

/// upsert 一条 `artifact_ref`：以 `(project_id, path, kind)` 为逻辑键。已存在则更新
/// `content_hash` 并 `version += 1`；否则插入 `version = 1`。切片 `content_hash`
/// 与 generation.jsonl 同源，供后续过期检测消费（留位不留债）。
pub fn upsert_artifact_ref(
    conn: &Connection,
    project_id: &str,
    path: &str,
    kind: &str,
    content_hash: &str,
) -> Result<(), CoreError> {
    let now = now_rfc3339();
    let existing: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, version FROM artifact_ref \
             WHERE project_id = ?1 AND path = ?2 AND kind = ?3",
            params![project_id, path, kind],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    match existing {
        Some((id, version)) => {
            conn.execute(
                "UPDATE artifact_ref SET content_hash = ?1, version = ?2, updated_at = ?3 \
                 WHERE id = ?4",
                params![content_hash, version + 1, now, id],
            )?;
        }
        None => {
            conn.execute(
                "INSERT INTO artifact_ref (project_id, path, kind, content_hash, version, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                params![project_id, path, kind, content_hash, now],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_central_db;
    use crate::domain::SopPhase;

    /// 建一个内存 DB（已迁移到最新 schema），种子一个 project + project_state 行。
    fn seeded_conn() -> (Connection, String) {
        let conn = open_central_db(":memory:").expect("open in-memory db");
        let project_id = "proj-state-test".to_string();
        let now = now_rfc3339();
        conn.execute(
            "INSERT INTO project (project_id, name, path, created_at, updated_at) \
             VALUES (?1, 'state-test', '/tmp/state-test', ?2, ?2)",
            params![project_id, now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO project_state (project_id, sop_phase, updated_at) VALUES (?1, ?2, ?3)",
            params![project_id, SopPhase::BriefDrafting.as_db_str(), now],
        )
        .unwrap();
        (conn, project_id)
    }

    fn current_phase(conn: &Connection, project_id: &str) -> String {
        conn.query_row(
            "SELECT sop_phase FROM project_state WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn advance_sop_phase_updates_row() {
        let (conn, pid) = seeded_conn();
        advance_sop_phase(&conn, &pid, SopPhase::VisionDrafting).unwrap();
        assert_eq!(current_phase(&conn, &pid), "vision_drafting");
    }

    #[test]
    fn gate_set_read_clear_round_trip() {
        let (conn, pid) = seeded_conn();
        assert_eq!(read_pending_gate(&conn, &pid).unwrap(), None);

        set_gate(&conn, &pid, "brief").unwrap();
        assert_eq!(read_pending_gate(&conn, &pid).unwrap().as_deref(), Some("brief"));

        clear_gate(&conn, &pid).unwrap();
        assert_eq!(read_pending_gate(&conn, &pid).unwrap(), None);
    }

    #[test]
    fn set_gate_preserves_other_cursor_keys() {
        let (conn, pid) = seeded_conn();
        // 预置一个无关游标键，验证关卡读改写不吞掉其他字段。
        conn.execute(
            "UPDATE project_state SET cursors = ?1 WHERE project_id = ?2",
            params![r#"{"block_cursor":3}"#, pid],
        )
        .unwrap();

        set_gate(&conn, &pid, "brief").unwrap();
        let map = read_cursors(&conn, &pid).unwrap();
        assert_eq!(map.get("block_cursor").and_then(Value::as_i64), Some(3));
        assert_eq!(map.get(PENDING_GATE_KEY).and_then(Value::as_str), Some("brief"));

        clear_gate(&conn, &pid).unwrap();
        let map = read_cursors(&conn, &pid).unwrap();
        assert_eq!(map.get("block_cursor").and_then(Value::as_i64), Some(3));
        assert!(map.get(PENDING_GATE_KEY).is_none());
    }

    #[test]
    fn chapter_load_and_status_update() {
        let (conn, pid) = seeded_conn();
        let now = now_rfc3339();
        conn.execute(
            "INSERT INTO volume (id, project_id, seq, arc_phase, updated_at) \
             VALUES ('ARC-1', ?1, 1, 'writing', ?2)",
            params![pid, now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chapter (id, project_id, volume_id, seq, status, updated_at) \
             VALUES ('CH-1', ?1, 'ARC-1', 1, 'void', ?2)",
            params![pid, now],
        )
        .unwrap();

        let chapter = load_chapter(&conn, "CH-1").unwrap();
        assert_eq!(chapter.status, ChapterState::Void);
        assert_eq!(chapter.volume_id, "ARC-1");
        assert_eq!(chapter.batch_id, None);

        update_chapter_status(&conn, "CH-1", ChapterState::Planned).unwrap();
        assert_eq!(load_chapter(&conn, "CH-1").unwrap().status, ChapterState::Planned);
    }

    #[test]
    fn insert_void_record_persists_row() {
        let (conn, pid) = seeded_conn();
        let now = now_rfc3339();
        conn.execute(
            "INSERT INTO volume (id, project_id, seq, arc_phase, updated_at) \
             VALUES ('ARC-1', ?1, 1, 'writing', ?2)",
            params![pid, now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chapter (id, project_id, volume_id, seq, status, updated_at) \
             VALUES ('CH-1', ?1, 'ARC-1', 1, 'void', ?2)",
            params![pid, now],
        )
        .unwrap();

        insert_void_record(&conn, &pid, "CH-1", Some("崩人设"), None).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM void_record WHERE chapter_id = 'CH-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn upsert_artifact_ref_inserts_then_bumps_version() {
        let (conn, pid) = seeded_conn();

        upsert_artifact_ref(&conn, &pid, "bible/style.md", "input", "hash-a").unwrap();
        let (hash, version): (String, i64) = conn
            .query_row(
                "SELECT content_hash, version FROM artifact_ref \
                 WHERE project_id = ?1 AND path = 'bible/style.md' AND kind = 'input'",
                params![pid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(hash, "hash-a");
        assert_eq!(version, 1);

        upsert_artifact_ref(&conn, &pid, "bible/style.md", "input", "hash-b").unwrap();
        let (hash, version, rows): (String, i64, i64) = conn
            .query_row(
                "SELECT content_hash, version, \
                 (SELECT COUNT(*) FROM artifact_ref WHERE project_id = ?1 AND path = 'bible/style.md') \
                 FROM artifact_ref WHERE project_id = ?1 AND path = 'bible/style.md' AND kind = 'input'",
                params![pid],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(hash, "hash-b");
        assert_eq!(version, 2);
        assert_eq!(rows, 1, "upsert 不应新增行");
    }
}
