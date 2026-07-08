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

## TASK-005 — 2026-07-08

变更文件：
- crates/mojian-core/src/engine/mod.rs（新增：Action / Verdict 枚举、RunState、纯函数 next_action、load_run_state、apply_generation、apply_decision + 单测）
- crates/mojian-core/src/state/mod.rs（新增：Chapter 视图 + advance_sop_phase / set_gate / clear_gate / read_pending_gate / load_chapter / update_chapter_status / insert_void_record / upsert_artifact_ref，cursors JSON 读改写 + 单测）
- crates/mojian-core/src/error.rs（修改：新增 MissingDecisionTarget 变体，供 VOID 缺 target 时报错）
- crates/mojian-core/src/lib.rs（修改：`pub mod engine; pub mod state;` + re-export 公共项）
- crates/mojian-core/tests/engine_loop.rs（新增：隔离临时 DB 集成测试，confirm / revise / void 三例）

实现摘要：落地 IMPL-4 状态机核心（选型 6-A run/decide 落 core::engine）。engine 以 `RunState{sop_phase, pending_gate}` → `Action` 的纯函数 next_action 守 AP-001：pending_gate 存在优先出 HumanGate（Generate 与关卡间预留客观检查步位、本迭代空过不写 check.jsonl，裁决①），否则 style_sampling/style_extracting→Advance（占位 no-op 直进）、brief_drafting→Generate{brief-agent}、更深 phase→Idle（诚实出口）。apply_generation 为每个输入切片 upsert artifact_ref（kind=input，content_hash 与 generation.jsonl 同源）后 set_gate(brief)。apply_decision 按 Verdict 收敛：CONFIRMED 清关卡+推进 vision_drafting；REVISE 清关卡+回退细粒度状态（brief 关卡回 brief_drafting，章节 target 回 skeleton_drafting，评论留待下次装配回喂）；VOID<CH> 写 void_record + chapter void→planned（裁决③ 最小语义，不做圣经级联 / 过期检测），target 缺失报 MissingDecisionTarget。state 模块全部走 rusqlite，关卡标记借 project_state.cursors JSON 的 pending_gate 字段（serde_json 读改写、保留其他游标键），无 schema 迁移、SCHEMA_VERSION 保持 1。

Build Verification：`cargo check -p mojian-core` 与 `cargo check --workspace` 0 error（快速校验）。未改 Cargo.toml、serde_json 为既有依赖、无新依赖、无 schema 迁移、未触发 devops.md 打包影响范围，故不触发打包校验。测试：`cargo test -p mojian-core --lib` 54 passed（含 engine 5 + state 6 新单测）；`MOJIAN_HOME=<tmp> cargo test -p mojian-core --test engine_loop` 3 passed（EXIT=0，隔离临时 central.db 登记项目 + 种子 volume/chapter 行 + 真实 apply_* + 回读 DB 断言）。
Builder Exit Criteria：9/9 通过。

已知风险：apply_decision 的 CONFIRMED 推进目标目前仅对 brief 关卡硬编码到 vision_drafting（本迭代唯一完整关卡），更深 SOP 关卡的推进映射待后续迭代接线时补；章节关卡（skeleton_review）不入 cursors 标记、REVISE 回退依赖调用方传 target，与 tech-design「章节关卡本身即关卡」一致。VOID 只记录 + 单章 void→planned，不做圣经级联 / 输入 hash 过期检测（留位不留债，归 ITER-003 消费本迭代写入的 artifact_ref.content_hash）。

## TASK-006 — 2026-07-08

变更文件：
- crates/mojian-cli/src/commands/run.rs（桩 → 真逻辑：定位项目 + sync_if_drifted + 循环 next_action + Generate 装配调 runner 落 generation.jsonl 置 brief 关卡 + Advance 顺推占位 phase + HumanGate/Idle 出口 + now_iso8601 std 时间戳）
- crates/mojian-cli/src/commands/decide.rs（桩 → 真逻辑：clap 解析 gate/verdict/target/--comment|--file + pending_gate 匹配校验 + append_decision + apply_decision）
- crates/mojian-cli/src/commands/status.rs（扩展：pending_gate 存在时追加卡点提示 REQ-008）
- crates/mojian-cli/tests/cli.rs（桩用例 run_and_decide_are_stubs → 端到端 run_decide_run_end_to_end，MOJIAN_HOME 隔离 + MOJIAN_CLAUDE_CMD 假命令驱动真实二进制）

实现摘要：收口 IMPL-4「生成闭环」的 CLI 通路（选型 6-A：状态机在 core，CLI 薄）。run 定位项目（读 mojian.toml 取 project_id）→ 复用 status 同口径的 ensure_master/open_db/sync_if_drifted 打开时 hash 覆盖 → 循环 engine::next_action：Advance 只在占位前置 phase（style_sampling→style_extracting→brief_drafting）纯推进；Generate → context::assemble_bundle → ClaudeCliRunner.run（基础命令读 MOJIAN_CLAUDE_CMD）→ 组 GenerationEvent（step/agent/spec_path/spec_hash/inputs/token_in/out/cost/ts）append_generation → apply_generation 置 brief 关卡 → 停并打印卡点；HumanGate 停机打印；Idle 正常退出（含 MAX_STEPS 空转保护）。generation.jsonl 的 inputs 把本步回喂的人类关卡评论（read_decision_comments gate=brief）记为输入切片（REQ-011），使评论文本随事件落盘。decide 用 clap 解析 gate/verdict/target 与互斥的 --comment/--file（--file 读文件内容作评论），校验 read_pending_gate 与请求 gate 一致（否则返回 CoreError::GateStateMismatch，非 0 退出、不 panic）后 append_decision + engine::apply_decision（CONFIRMED 推进 vision_drafting / REVISE 回退 brief_drafting 回喂 / VOID 章节最小语义）。status 在既有 project/phase 后按 read_pending_gate 追加「卡在 <gate> 关卡 / 等待判定：CONFIRMED|REVISE|VOID」。run/decide 复用 now_iso8601（std SystemTime + Hinnant days-from-civil 逆算法生成 RFC3339 UTC 秒级时间戳，避免为 CLI 新增 time 依赖）。CLI 层不新增任何 Cargo 依赖、不改 core。

Build Verification：`cargo check --workspace` 0 error（快速校验）；本次改动仅动 mojian-cli 源码与集成测试、未改 Cargo.toml/嵌入资产，未触发 devops.md 打包影响范围，仍按 REQ-002 验收口执行打包校验 `cargo build --workspace` 0 error/0 warning。测试：`cargo test -p mojian-cli --test cli` 5 passed（含新 run_decide_run_end_to_end，MOJIAN_HOME 隔离 + MOJIAN_CLAUDE_CMD 假脚本真实 spawn，不触达真实 claude）；`cargo test --workspace` 全绿无回归。手动 E2E 复核 new→run(停 brief)→status(显卡点)→decide REVISE(写 decision.jsonl)→run(评论「钩子太弱」回喂进 generation.jsonl 第 2 行 inputs)→decide CONFIRMED→run(vision_drafting Idle 不再卡 brief)→decide 关卡不匹配(exit 1 + 关卡状态不匹配 无 panic)。
Builder Exit Criteria：6/6 通过。

已知风险：status 卡点提示当前仅覆盖 SOP① 顺序关卡（project_state.cursors.pending_gate）；章节级 skeleton_review 卡点显示与章节 VOID/REVISE 判定按 tech-design.md「端到端验收深度」走 core 单元/集成测覆盖，未塞进 CLI E2E（需 volume/batch/chapter 行种子）。decide 的 gate 校验以 pending_gate 为准，仅对 SOP① 顺序关卡生效，与本迭代 CLI 面契约一致。now_iso8601 为 std 自算 UTC 秒级时间戳（与 core::now_rfc3339 的 time crate 实现同语义、不同实现），仅用于日志 ts 字段、不参与任何断言。
