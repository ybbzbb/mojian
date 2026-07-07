use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("无法解析客户端数据目录：未设置 MOJIAN_HOME，且平台标准目录与用户主目录均不可用")]
    DataDirUnresolved,

    #[error("I/O 错误（路径 {path:?}）：{source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("未知的 {kind} DB 文本值：{value:?}")]
    UnknownDomainValue { kind: &'static str, value: String },

    #[error("中央 DB 操作失败：{0}")]
    Db(#[from] rusqlite::Error),
}
