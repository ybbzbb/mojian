use std::path::PathBuf;

use crate::error::CoreError;

const HOME_ENV: &str = "MOJIAN_HOME";
const APP_NAME: &str = "mojian";
const FALLBACK_SUBDIR: &str = ".mojian";

const CENTRAL_DB_FILE: &str = "central.db";
const SPEC_SUBDIR: &str = "spec";
const LOGS_SUBDIR: &str = "logs";

/// 客户端数据目录分层解析：
/// `MOJIAN_HOME` 环境变量 → 平台标准目录（`directories::ProjectDirs`，
/// Linux `$XDG_DATA_HOME/mojian`、macOS `~/Library/Application Support/mojian`）
/// → 兜底 `~/.mojian/`。
pub fn data_dir() -> Result<PathBuf, CoreError> {
    if let Some(raw) = std::env::var_os(HOME_ENV) {
        if !raw.is_empty() {
            return Ok(PathBuf::from(raw));
        }
    }

    if let Some(dirs) = directories::ProjectDirs::from("", "", APP_NAME) {
        return Ok(dirs.data_dir().to_path_buf());
    }

    if let Some(base) = directories::BaseDirs::new() {
        return Ok(base.home_dir().join(FALLBACK_SUBDIR));
    }

    Err(CoreError::DataDirUnresolved)
}

/// 客户端中央 SQLite 数据库路径：`<data_dir>/central.db`。
pub fn central_db_path() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join(CENTRAL_DB_FILE))
}

/// SPEC 主副本目录：`<data_dir>/spec`。
pub fn spec_master_dir() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join(SPEC_SUBDIR))
}

/// 日志根目录：`<data_dir>/logs`（按 `project_id` 分区置于其下）。
pub fn logs_dir() -> Result<PathBuf, CoreError> {
    Ok(data_dir()?.join(LOGS_SUBDIR))
}
