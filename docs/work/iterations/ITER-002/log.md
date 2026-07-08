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

## TASK-002 — 2026-07-08

变更文件：
- crates/mojian-core/src/log/mod.rs（新增）
- crates/mojian-core/src/lib.rs（修改：`pub mod log;` + re-export log 公共项）
- crates/mojian-core/tests/log_jsonl.rs（新增：集成测试真实写盘到隔离 data_dir）

实现摘要：新增 `mojian-core::log` 模块，作为 REQ-006 / REQ-010 / REQ-011 的 JSONL 日志基座。定义 `GenerationEvent`（step/agent/spec_path/spec_hash/inputs/token_in/token_out/cost/ts，inputs 为 `InputSlice{path/anchor/content_hash}` 列表）与 `DecisionEvent`（gate/verdict/target/comment/ts），均 `serde::{Serialize,Deserialize}`，空 `Option` 以 `skip_serializing_if` 省略。`append_generation` / `append_decision` 经私有 `append_jsonl` 泛型辅助：`fs::create_dir_all` 确保 `<data_dir>/logs/{project_id}/` 存在，`OpenOptions::append` + `writeln!` 单行追加、只增不改。`read_decision_comments(project_id, gate, target)` 逐行反序列化 `decision.jsonl`，经纯函数 `pick_comment` 过滤（gate 相同 + comment 非空 + target 命中：查询 target 为 `None` 取全部、事件 target 为空视为全局评论），按写入顺序返回；文件不存在返回空 `Vec`。不写 `check.jsonl`（裁决①，归 ITER-003）。lib.rs 导出全部公共项。

Build Verification：`cargo check` 0 error（快速校验）；未触及打包文件范围（无 Cargo.toml / 依赖变更），跳过打包校验。`cargo test -p mojian-core --test log_jsonl` EXIT=0（3 passed）；`cargo test -p mojian-core log`（log 模块单测）5 passed。
Builder Exit Criteria：9/9 通过。

已知风险：无。集成测试将同一 `MOJIAN_HOME`（temp/mojian-log-test-{pid}）用唯一 `project_id` 分区，对并行执行安全。
