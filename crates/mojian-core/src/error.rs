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

    #[error("生成子进程执行失败（命令 {command:?}，退出码 {code:?}）：{stderr}")]
    SubprocessFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("JSON 解析失败：{0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("输入契约 manifest 非法（路径 {path:?}）：{reason}")]
    ManifestInvalid { path: PathBuf, reason: String },

    #[error("符号引用无法解析（符号 {symbol:?}）：{reason}")]
    SymbolUnresolved { symbol: String, reason: String },

    #[error("关卡状态不匹配：期望处于关卡 {expected:?}，实际为 {actual:?}")]
    GateStateMismatch { expected: String, actual: String },
}
