//! 中央 SQLite（`central.db`）访问的唯一入口。
//!
//! `open_central_db` 是所有 DB 访问的门面：打开连接 → 跑迁移 → 返回 `Connection`。
//! 任何 DB 操作都应经此入口取得连接，确保打开即已迁移到最新 schema。

mod migrate;
mod schema;

pub use schema::SCHEMA_VERSION;

use std::path::Path;

use rusqlite::Connection;

use crate::error::CoreError;

/// 打开客户端中央 DB 并跑迁移，返回已就绪的连接。
///
/// 全新库会在单事务内建全部 12 表并写入版本戳；已有库按 `schema_meta.schema_version`
/// 升级；同一路径重复打开是幂等的（不重复建表、不报错）。
pub fn open_central_db<P: AsRef<Path>>(path: P) -> Result<Connection, CoreError> {
    let mut conn = Connection::open(path.as_ref())?;
    migrate::run_migrations(&mut conn)?;
    Ok(conn)
}
