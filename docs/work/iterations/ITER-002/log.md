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

## TASK-003 — 2026-07-08

变更文件：
- crates/mojian-core/src/sdk/mod.rs（新增：Bundle / SdkResponse / GenerationRunner trait + #[cfg(test)] FakeRunner 单测）
- crates/mojian-core/src/sdk/claude_cli.rs（新增：ClaudeCliRunner）
- crates/mojian-core/src/lib.rs（修改：`pub mod sdk;` + re-export `Bundle` / `ClaudeCliRunner` / `GenerationRunner` / `SdkResponse`）
- crates/mojian-core/tests/sdk_runner.rs（新增：集成测试，假命令真实 spawn）

实现摘要：新增 `mojian-core::sdk` 模块，落地 REQ-003/004/005 的 SDK 调用抽象（选型 1-A / 2-A）。`Bundle` 承载一次生成的五字段：`agent`（部署 agent 相对路径）、`spec_slice`、`inputs`（内联 SSOT 结构化参数 + decision 回喂评论）、`write_scope: Vec<PathBuf>`、`output_contract`。`SdkResponse` 手写 `Deserialize`：经中间 `Raw` 结构（`total_cost_usd` serde rename → `cost`；嵌套 `usage.input_tokens` / `usage.output_tokens` → `usage_in` / `usage_out`），全部字段 `Option` + `#[serde(default)]` 容错，字段缺失不报错。`trait GenerationRunner { fn run(&self, &Bundle) -> Result<SdkResponse, CoreError> }` 让状态机与生成 IO 解耦。默认实现 `ClaudeCliRunner`（`claude_cli.rs`）用 `std::process::Command` 拼 `claude -p <prompt> --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope 逐项>`；基础命令名读 `MOJIAN_CLAUDE_CMD`（`std::env::var`，缺省 `"claude"`），在项目目录内运行（`current_dir`），`output()` 一次抓 stdout/stderr/exit；非 0 退出 → `CoreError::SubprocessFailed { command, code, stderr }`，stdout JSON 解析失败 → `CoreError::JsonParse`（`?` 承接，TASK-001 预置变体）。prompt 由五字段确定性拼装。双层注入验证：core 单测走 `FakeRunner` 不 spawn，CLI/集成走 `MOJIAN_CLAUDE_CMD` 假命令。

Build Verification：`cargo check -p mojian-core` 0 error（快速校验）；未触及打包文件范围（无 Cargo.toml / 依赖变更，`serde_json` 已由 TASK-001 就位），`cargo check --workspace` 亦 0 error（含 mojian-cli）。测试：`cargo test -p mojian-core --test sdk_runner` 2 passed（假命令真实 spawn：固定 JSON 解析 + 非 0 退出 → Err）；`cargo test -p mojian-core --lib sdk` 4 passed（SdkResponse 全字段 / 缺字段 / 半份 usage 容错 + FakeRunner 注入）。
Builder Exit Criteria：8/8 通过。

已知风险：无。集成测试对进程级 `MOJIAN_CLAUDE_CMD` 以 `static ENV_LOCK: Mutex` 串行化读写，避免同二进制内并行测试互踩；假命令脚本为 unix `#!/bin/sh` + `chmod 0755`（平台 darwin，符合 devops.md 构建环境）。真实 `claude --output-format json` schema 差异按 tech-design 由 mock 隔离，留待 ITER-003 实测收口。

## TASK-004 — 2026-07-08

变更文件：
- crates/mojian-core/src/context/manifest.rs（新增：InputManifest TOML sidecar 模型 + read_input_manifest + write_scope 推导 + 单测）
- crates/mojian-core/src/context/symbol.rs（新增：`<source>.<selector>[:{params}][#anchor]` 手写解析 + 占位代入 + source→基路径映射 + 单测）
- crates/mojian-core/src/context/slice.rs（新增：段级 `#anchor` 抽取 / 整文件切片 + blake3 content_hash + 单测）
- crates/mojian-core/src/context/assemble.rs（新增：assemble_bundle 全链 + render_inputs + default_sources + 单测）
- crates/mojian-core/src/context/mod.rs（新增：子模块聚合 + 导出）
- crates/mojian-core/src/lib.rs（修改：`pub mod context;` + re-export `assemble_bundle`）
- crates/mojian-cli/assets/spec/.claude/agents/brief-agent.md（新增：占位提示词，含稳定小标题 `## inputs` / `## output`）
- crates/mojian-cli/assets/spec/.claude/agents/brief-agent.manifest.toml（新增：占位输入契约，inputs = 整文件 workspace + 段级 spec.brief-agent#inputs，write = creative/creative-brief.md，gate = brief）
- crates/mojian-core/tests/context_assemble.rs（新增：端到端集成测试）

实现摘要：落地 IMPL-3 切片装配全链（选型 3-A TOML sidecar / 选型 5-A 手写符号解析器）。`manifest.rs` 用既有 `toml` + `serde` 反序列化 sidecar，非法 → `CoreError::ManifestInvalid`。`symbol.rs` 按 `#` / `:` / `.` 切分文法，支持纯整文件、`.selector`、`:{params}`、`#anchor` 四类；占位代入支持 `{ident}` 直查与 `{ident±N}` 整数算术（`{ch-1}`）、逗号多项（`{arc_id,batch}`）；无法解析 → `CoreError::SymbolUnresolved`；source 经映射表解析基路径（`workspace`→`CLAUDE.md`、`spec`→`.claude/agents`），未登记 source 回退为名字即基路径（兼容 `bible.style#skeleton`）。`slice.rs` 段级切片依 markdown 标题正文/slug 匹配抽取到下一同级或更高级标题（不含），整文件读全文；两者 `blake3::hash` 出 hex content_hash；文件缺失 → `Io`，锚点未命中 → `SymbolUnresolved`。`assemble.rs` 的 `assemble_bundle(conn, project_id, project_dir, manifest_path)` 串「读 manifest → 由 `load_project_state` 取当前状态构造占位表 → 解析符号 → 切片 → 读 agent 提示词作 spec_slice → `log::read_decision_comments` 回喂人类评论 → `write:` 推导 write_scope → 组五字段 Bundle」；inputs 文本内联每片 path/anchor/content_hash + 内容，末尾附本关卡人类评论（REQ-011）。占位 brief-agent 步骤资产落 `.claude/agents`，随现有部署 / hash 覆盖生效（无需改 deploy.rs），其 inputs 指向部署后必然存在的 `CLAUDE.md` 与自身，assemble 无需手工种子即可解析。

Build Verification：`cargo check --workspace` 0 error（快速校验）；改动新增了嵌入 SPEC 树资产（经 include_dir 编入 mojian-core，属打包影响范围），执行打包校验 `cargo build --workspace` 0 error。测试：`cargo test -p mojian-core --lib context` 25 passed；`cargo test -p mojian-core --test context_assemble` 2 passed（EXIT=0，隔离 MOJIAN_HOME + 临时项目部署占位 SPEC + 种子 DB 行 + 写 decision.jsonl + 真实 assemble_bundle）；`cargo test --workspace` 全绿无回归（mojian-core lib 43 / 各集成测 + mojian-cli 5）。
Builder Exit Criteria：8/8 通过。

已知风险：符号 source→基路径映射当前仅登记占位步骤所需 `workspace` / `spec` 两项，真实 SSOT 源（bible/outline/plan 等）走回退规则（名字即基路径），后续迭代接章节/批次推进时在 `default_sources` / `state_map` 补键——留位不留债，与 tech-design「占位简化」一致。params 目前仅影响路径 stem（`chapters-{batch}`），不驱动 DB 行选择，占位步骤未用；集成测试将同一 MOJIAN_HOME 以唯一 project_id 分区、DB 开在项目内独立 central.db，对并行执行安全。
