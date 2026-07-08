# Build Log — ITER-002

## TASK-001 — 2026-07-08

变更文件：
- Cargo.toml（修改：`[workspace.dependencies]` 追加 `serde_json = "1"`）
- crates/mojian-core/Cargo.toml（修改：`[dependencies]` 引入 `serde_json = { workspace = true }`）
- crates/mojian-core/src/error.rs（修改：`CoreError` 新增 5 个变体）

实现摘要：为本迭代打地基。在 workspace 引入唯一新增依赖 `serde_json`（选型 4），并在 `mojian-core::CoreError` 预置后续 sdk/log/context/engine 模块需要的错误变体——`SubprocessFailed { command, code, stderr }`（子进程失败，携命令名 + exit + stderr 足以定位）、`JsonParse(#[from] serde_json::Error)`（JSON 解析失败，直接承接底层错误）、`ManifestInvalid { path, reason }`、`SymbolUnresolved { symbol, reason }`、`GateStateMismatch { expected, actual }`。命名 PascalCase，`#[error(...)]` 中文消息对齐既有风格。

Build Verification：`cargo check` 0 error；`cargo build --workspace` Finished（退出码 0）。
Builder Exit Criteria：7/7 通过。
辅助验证：`cargo test -p mojian-core --lib` 9 passed / 0 failed；`cargo tree -p mojian-core` 含 `serde_json v1.0.150`，`grep -E 'serde_yaml|tokio|reqwest'` 无输出。

已知风险：无。Cargo.lock 因新增依赖自动更新（生成文件，非 Allowed Files 源码，随依赖变更不可避免）。
