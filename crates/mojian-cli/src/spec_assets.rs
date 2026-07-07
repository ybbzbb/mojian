//! 编译期嵌入的占位 SPEC 主副本载荷树。
//!
//! `assets/spec/` 经 `include_dir` 编译进二进制，首次运行时由 core 的 `ensure_master`
//! 写出到 `<data_dir>/spec/`，形成客户端权威主副本。CLI 负责嵌入并注入，core 侧的
//! bootstrap/部署函数以参数化的 `&Dir` 接口接收，不硬编码资产来源。

use include_dir::{include_dir, Dir};

/// 占位 SPEC 主副本载荷树（源在 `crates/mojian-cli/assets/spec/`）。
pub static SPEC_ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets/spec");
