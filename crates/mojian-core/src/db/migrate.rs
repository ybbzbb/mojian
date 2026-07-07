//! 自研迁移运行器，键于 `schema_meta.schema_version`（非 PRAGMA user_version）。
//!
//! storage.md 明确要求以 `schema_meta.schema_version` 驱动升级；`rusqlite_migration` /
//! `refinery` 均用 `user_version`，故不采用。迁移按「有序编号步骤 + 事务 + 失败回滚」实现：
//! 全新库从版本 0 起跑到 `SCHEMA_VERSION`；已有库读当前版本，只跑更高编号的步骤。

use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::error::CoreError;

use super::schema::{self, SCHEMA_VERSION};

/// 单个有序迁移步骤：执行后使 schema 达到 `version`。
struct Migration {
    version: i64,
    sql: &'static str,
}

/// 有序迁移步骤表（按 `version` 升序）。v1 = 建全部 12 表；后续版本在此追加。
const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: schema::V1_SCHEMA_SQL,
}];

/// 跑迁移：`PRAGMA foreign_keys = ON` → 事务内按有序步骤升级 → 提交（失败自动回滚）。
///
/// - 全新库（无 `schema_meta` 表）：从版本 0 跑全部步骤，并 `INSERT` 版本戳。
/// - 已有库：读 `schema_meta.schema_version`，只跑更高编号步骤，并 `UPDATE` 版本戳。
/// - 版本已是最新：no-op，不重复建表、不报错。
pub fn run_migrations(conn: &mut Connection) -> Result<(), CoreError> {
    // PRAGMA foreign_keys 只在事务外生效，故先于事务设置。
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let tx = conn.transaction()?;
    let from = current_version(&tx)?;
    let start = from.unwrap_or(0);

    for step in MIGRATIONS.iter().filter(|m| m.version > start) {
        tx.execute_batch(step.sql)?;
    }

    if from.is_none() {
        tx.execute(
            "INSERT INTO schema_meta (schema_version) VALUES (?1)",
            [SCHEMA_VERSION],
        )?;
    } else if start < SCHEMA_VERSION {
        tx.execute("UPDATE schema_meta SET schema_version = ?1", [SCHEMA_VERSION])?;
    }

    tx.commit()?;
    Ok(())
}

/// 读当前 schema 版本；`schema_meta` 表不存在（全新库）时返回 `None`。
fn current_version(tx: &Transaction) -> Result<Option<i64>, CoreError> {
    let table_exists: i64 = tx.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_meta'",
        [],
        |row| row.get(0),
    )?;
    if table_exists == 0 {
        return Ok(None);
    }
    let version = tx
        .query_row("SELECT schema_version FROM schema_meta LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?;
    Ok(version)
}
