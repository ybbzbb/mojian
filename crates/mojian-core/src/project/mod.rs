//! 项目登记与身份标记读写。
//!
//! DB 侧：`register_project`（单事务登记 `project` + `project_state` 两行）、
//! `load_project_state`（按 `project_id` 读回 SOP phase）、`update_project_spec`
//! （回填部署得到的 `spec_version` / `spec_hash`）。文件侧：`manifest` 子模块读写 `mojian.toml`。

pub mod manifest;

pub use manifest::{read_manifest, write_manifest, ProjectManifest};

use std::path::Path;

use rusqlite::{params, Connection};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::SopPhase;
use crate::error::CoreError;

/// 当前 UTC 时刻的 RFC3339 文本，写入 TEXT 时间列。
fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 formatting of the current UTC time is infallible")
}

/// 把 `path` 规整为绝对路径（不解析符号链接，不要求目标存在）。
fn to_absolute(path: &Path) -> Result<String, CoreError> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| CoreError::Io {
                path: path.to_path_buf(),
                source,
            })?
            .join(path)
    };
    Ok(abs.to_string_lossy().into_owned())
}

/// 登记一个新项目：在单事务内插入 `project` 行与 `project_state` 行。
///
/// 生成 UUID v4 `project_id`；`path` 以绝对路径存入；`project_state.sop_phase`
/// 初值为 `style_sampling`（SOP① 首阶段）。返回新 `project_id`。
pub fn register_project(
    conn: &mut Connection,
    name: &str,
    path: &Path,
) -> Result<String, CoreError> {
    let project_id = Uuid::new_v4().to_string();
    let abs_path = to_absolute(path)?;
    let now = now_rfc3339();

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO project (project_id, name, path, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![project_id, name, abs_path, now],
    )?;
    tx.execute(
        "INSERT INTO project_state (project_id, sop_phase, updated_at) VALUES (?1, ?2, ?3)",
        params![project_id, SopPhase::StyleSampling.as_db_str(), now],
    )?;
    tx.commit()?;

    Ok(project_id)
}

/// 按 `project_id` 读回当前 SOP phase。无此项目返回 `CoreError::Db`
/// （`QueryReturnedNoRows`）；DB 文本非法则返回 `CoreError::UnknownDomainValue`。
pub fn load_project_state(conn: &Connection, project_id: &str) -> Result<SopPhase, CoreError> {
    let phase_text: String = conn.query_row(
        "SELECT sop_phase FROM project_state WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )?;
    SopPhase::try_from(phase_text.as_str())
}

/// 回填 `project` 行的部署身份：`spec_version` / `spec_hash` / `updated_at`。
pub fn update_project_spec(
    conn: &Connection,
    project_id: &str,
    spec_version: &str,
    spec_hash: &str,
) -> Result<(), CoreError> {
    let now = now_rfc3339();
    conn.execute(
        "UPDATE project SET spec_version = ?1, spec_hash = ?2, updated_at = ?3 \
         WHERE project_id = ?4",
        params![spec_version, spec_hash, now, project_id],
    )?;
    Ok(())
}
