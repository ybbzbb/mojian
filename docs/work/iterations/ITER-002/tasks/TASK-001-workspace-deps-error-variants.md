# TASK-001 workspace 依赖 serde_json + error 变体扩展

- iteration: ITER-002
- status: done
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-08

## Goal

为本迭代打地基：在 workspace 引入本迭代唯一新增依赖 `serde_json`（供 JSON 解析 + JSONL 序列化），并在 `mojian-core` 的 `CoreError` 中预置本迭代后续模块需要的错误变体。产出是可编译的依赖声明与错误类型扩展，供 log / sdk / context / engine 模块直接消费。

## Allowed Files

- Cargo.toml（workspace 根，仅 `[workspace.dependencies]` 追加）
- crates/mojian-core/Cargo.toml（`[dependencies]` 引入 serde_json）
- crates/mojian-core/src/error.rs
- 禁止：crates/mojian-cli/**
- 禁止：crates/mojian-core/src/{db,domain,project,spec,paths}/**（本任务不动既有模块）
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#选型对比 — 选型 4（`serde_json` 为唯一新增依赖，不引入 serde_yaml/async/HTTP/LLM SDK）
- docs/work/iterations/ITER-002/tech-design.md#涉及模块 — `error.rs` 需新增：子进程失败 / JSON 解析失败 / manifest 非法 / 符号无法解析 / 关卡状态不匹配

## Builder Exit Criteria

- [x] `Cargo.toml`（workspace）`[workspace.dependencies]` 追加 `serde_json = "1"`；不新增其它依赖（无 serde_yaml / tokio / reqwest 等）。
- [x] `crates/mojian-core/Cargo.toml` `[dependencies]` 引入 `serde_json = { workspace = true }`。
- [x] `crates/mojian-core/src/error.rs` 的 `CoreError` 至少新增 5 个变体，语义对齐 tech-design.md「涉及模块」表：子进程执行失败、JSON 解析失败、manifest 非法、符号无法解析、关卡状态不匹配（命名 PascalCase，`#[error(...)]` 中文消息与既有风格一致）。
- [x] JSON 解析失败变体以 `#[from] serde_json::Error` 或携带 `#[source]` 承接底层错误；子进程失败变体携带命令名 / exit 信息足以定位。
- [x] `cargo check` 0 error；`cargo build --workspace` 成功。
- [x] 既有单元测试不回归：`cargo test -p mojian-core --lib` 全绿。
- [x] 命名遵循 docs/naming.md（类型 PascalCase、常量 UPPER_SNAKE_CASE）。

## QA Verification

- [x] `cargo build --workspace` 退出码 0（真实构建产物，devops.md Build Verification 口径）。
- [x] `cargo tree -p mojian-core | grep serde_json` 输出含 `serde_json v1`，确认依赖已进依赖图。
- [x] `cargo tree -p mojian-core | grep -E 'serde_yaml|tokio|reqwest'` 无输出，确认未引入被禁依赖。

## Dependencies

- 前置任务：无

## Log

- 2026-07-07 [planning-agent] created status=ready
- 2026-07-08 [builder-agent] status ready → in_progress：认领任务
- 2026-07-08 [builder-agent] status in_progress → reviewing：workspace 加 serde_json="1" + mojian-core 引入、error.rs 新增 5 个 CoreError 变体（SubprocessFailed/JsonParse #[from] serde_json::Error/ManifestInvalid/SymbolUnresolved/GateStateMismatch）；cargo check 0 error、cargo build --workspace 成功、cargo test -p mojian-core --lib 9 passed；cargo tree 确认 serde_json v1 进图、无 serde_yaml/tokio/reqwest。Build Verification 与 Builder Exit Criteria 全部通过
- 2026-07-08 [qa-agent] status reviewing → done：QA Verification 3/3 通过（cargo build --workspace EXIT=0；cargo tree 含 serde_json v1.0.150；serde_yaml/tokio/reqwest grep 无输出；附加 cargo test -p mojian-core --lib 9 passed / 0 failed）
