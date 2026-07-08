# TASK-003 sdk 模块：GenerationRunner trait + Bundle + ClaudeCliRunner

- iteration: ITER-002
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

新增 `mojian-core::sdk` 模块：定义 `GenerationRunner` trait、五字段 `Bundle`（REQ-003）、`SdkResponse`（REQ-005），并实现默认 `ClaudeCliRunner`——以 `std::process::Command` 拼无头 `claude` 调用（REQ-004），基础命令名读自 `MOJIAN_CLAUDE_CMD`（缺省 `claude`），用 `serde_json` 解析 stdout JSON。外部命令可注入替换（硬约束）：CLI E2E 走 `MOJIAN_CLAUDE_CMD` 假命令，core 测走 `FakeRunner` trait 实现（不 spawn 进程）。

## Allowed Files

- crates/mojian-core/src/sdk/mod.rs（新增：trait / Bundle / SdkResponse）
- crates/mojian-core/src/sdk/claude_cli.rs（新增：ClaudeCliRunner）
- crates/mojian-core/src/lib.rs（仅追加 `pub mod sdk;` 与 sdk 公共项 re-export）
- crates/mojian-core/src/error.rs（如需补子进程 / JSON 变体的细节；变体主体已在 TASK-001 预置）
- crates/mojian-core/tests/sdk_runner.rs（新增集成测试）
- 禁止：crates/mojian-core/src/{log,context,engine,state}/**
- 禁止：crates/mojian-cli/**
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#选型对比 — 选型 1-A（trait + MOJIAN_CLAUDE_CMD 双层注入）、选型 2-A（std::process::Command）
- docs/work/iterations/ITER-002/tech-design.md#采用方案 — `SdkResponse` 字段：result / cost(total_cost_usd) / usage_in(usage.input_tokens) / usage_out(usage.output_tokens)，serde rename + Option 容错
- docs/tech-design/engine.md#SDK 调用：Rust → 无头 claude 子进程 — 调用形态：`claude -p <...> --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope>`
- docs/tech-design/engine.md#bundle：一次 SDK 调用喂进去的东西 — 五字段语义

## Builder Exit Criteria

- [ ] `Bundle` 结构体含五字段：`agent`（部署 agent 相对路径）、`spec_slice`、`inputs`（切片后 SSOT 结构化参数，含回喂评论）、`write_scope: Vec<PathBuf>`、`output_contract`（对齐 engine.md bundle 表 + REQ-003）。
- [ ] `SdkResponse` 结构体：`result: String`、`cost: Option<f64>`（serde rename `total_cost_usd`）、`usage_in: Option<u64>` / `usage_out: Option<u64>`（映射 `usage.input_tokens` / `usage.output_tokens`），字段以 serde rename + Option 容错解析（REQ-005）。
- [ ] `trait GenerationRunner { fn run(&self, bundle: &Bundle) -> Result<SdkResponse, CoreError>; }`。
- [ ] `ClaudeCliRunner` 实现 `GenerationRunner`：用 `std::process::Command` 拼 `claude -p <prompt> --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope 逐项>`；基础命令名读 `MOJIAN_CLAUDE_CMD`（`std::env::var`，缺省 `"claude"`）；在项目目录内运行（`current_dir`）；`output()` 抓 stdout/stderr/exit（对齐 REQ-004、engine.md 调用形态）。
- [ ] 子进程非 0 退出 → 返回子进程失败错误（含 exit 与 stderr 摘要）；stdout JSON 解析失败 → 返回 JSON 解析错误变体（均用 TASK-001 预置变体）。
- [ ] `lib.rs` 导出 `GenerationRunner` / `ClaudeCliRunner` / `Bundle` / `SdkResponse`；`cargo check` 0 error。
- [ ] 单元测试（`#[cfg(test)]`）提供 `FakeRunner`（不 spawn 进程）并覆盖：`SdkResponse` 从形如 `{"result":"...","total_cost_usd":0.01,"usage":{"input_tokens":10,"output_tokens":20}}` 的 JSON 正确解析；缺 cost/usage 字段时 Option 为 None 不报错。
- [ ] 命名遵循 docs/naming.md。

## QA Verification

- [ ] `cargo test -p mojian-core --test sdk_runner` 退出码 0（集成测试通过 `MOJIAN_CLAUDE_CMD` 指向一个测试内生成的假命令脚本，`ClaudeCliRunner` 真实 spawn 该子进程，验证「外部命令可替换」硬约束、不触达真实 claude）。
- [ ] 集成测试须断言：假命令输出固定 JSON 时，`ClaudeCliRunner::run` 返回的 `SdkResponse.result` 等于假命令产出的 result 文本，且 `cost` / `usage_in` / `usage_out` 被正确解析。
- [ ] 集成测试须断言：假命令以非 0 退出码结束时，`run` 返回 `Err`（子进程失败变体），不 panic。

## Dependencies

- 前置任务：TASK-001

## Log

- 2026-07-07 [planning-agent] created status=planned（依赖 TASK-001）
