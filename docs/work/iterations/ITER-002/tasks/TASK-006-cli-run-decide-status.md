# TASK-006 CLI run/decide/status 收口 + run→decide→run 端到端

- iteration: ITER-002
- status: reviewing
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-08

## Goal

把 CLI `run.rs` / `decide.rs` 从桩转真逻辑、扩展 `status.rs` 显卡点，端到端收口本迭代「生成闭环」（IMPL-4）。`run` 定位项目 → 循环 `engine::next_action` → Generate 执行（assemble → 调 runner → append_generation → 置关卡）→ 撞人工关卡即停（REQ-007）；`decide` 解析三判定 → append_decision → apply_decision（REQ-009/010）；`status` 显示卡在哪个关卡 / 等什么决定（REQ-008）。用 `MOJIAN_CLAUDE_CMD` 指向假命令端到端跑通一次 `run → decide → run`（REQ-012，裁决②）。

## Allowed Files

- crates/mojian-cli/src/commands/run.rs（桩 → 真逻辑）
- crates/mojian-cli/src/commands/decide.rs（桩 → 真逻辑）
- crates/mojian-cli/src/commands/status.rs（扩展显卡点）
- crates/mojian-cli/tests/cli.rs（更新：替换 run/decide 桩用例为端到端用例）
- 禁止：crates/mojian-cli/src/commands/{mod.rs,new.rs}（分发与 new 不改）
- 禁止：crates/mojian-cli/src/{main.rs,spec_assets.rs}
- 禁止：crates/mojian-core/**（复用 core 公共 API，不改 core 实现）
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#API 变更 — `mojian run` / `mojian decide` / `mojian status` 命令面行为与分支
- docs/work/iterations/ITER-002/tech-design.md#依赖与风险 — 端到端验收深度：CLI E2E 覆盖 brief 通路（mock SDK，REVISE 带评论演示 REQ-011 回喂 / CONFIRMED 演示推进）；章节级 VOID/REVISE 走 core 测不塞 CLI E2E
- docs/work/iterations/ITER-002/requirements.md — 裁决②（最小验收深度 = 一次 run→decide→run，用注入 mock SDK 端到端测，不要求真实创作物 / 多关卡链路）
- crates/mojian-core/src/{engine,state,context,log,sdk}/ — 复用的 core 公共 API（TASK-002~005 产出）
- crates/mojian-cli/src/commands/status.rs — ITER-001 已有的「project / phase」输出（本任务在其上扩展）
- crates/mojian-cli/tests/cli.rs — 既有 E2E 骨架（unique_dir / run_mojian / MOJIAN_HOME 隔离模式），复用其模式

## Builder Exit Criteria

- [x] `run.rs`：定位项目（读 `mojian.toml` 取 `project_id`）→ 复用 `sync_if_drifted` 打开时 hash 覆盖 → 循环 `engine::next_action`：`Advance` 纯推进 `sop_phase`；`Generate` → `context::assemble_bundle` → `runner.run(bundle)`（默认 `ClaudeCliRunner`）→ `log::append_generation` → `engine::apply_generation` 置关卡 → 停；`HumanGate` → 停机打印卡点；`Idle` → 正常退出。
- [x] `decide.rs`：解析 `<gate> <verdict> [target] [--comment "..." | --file <path>]`（用 clap；verdict ∈ CONFIRMED|REVISE|VOID）→ 校验当前确在 `<gate>`（否则非 0 退出，用关卡状态不匹配错误）→ `log::append_decision` → `engine::apply_decision`；`--file` 读文件内容作评论补充。
- [x] `status.rs`：在既有「project / phase」输出基础上，若 `project_state.cursors` 含 `pending_gate`，追加打印「卡在 `<gate>` 关卡 / 等待判定：CONFIRMED|REVISE|VOID」（REQ-008）。（章节级 `skeleton_review` 卡点走 core 测覆盖，非 CLI E2E，见 tech-design.md「端到端验收深度」。）
- [x] `cargo check` 0 error；`cargo build --workspace` 成功。
- [x] `cli.rs` 中原 `run_and_decide_are_stubs` 桩用例被替换为真实端到端用例 `run_decide_run_end_to_end`（不再断言「stub，将在 ITER-002 实现」字样）。
- [x] 命名遵循 docs/naming.md；CLI 层保持薄（解析参数 → 调 core → 打印），状态机映射由 `engine::next_action` 决定，命令处理函数不承载状态机逻辑。

## QA Verification

- [ ] 端到端 run→decide→run（裁决②，mock SDK）：`cargo test -p mojian-cli --test cli` 退出码 0；用例须以 `MOJIAN_HOME=<隔离目录>` + `MOJIAN_CLAUDE_CMD=<假命令>` 驱动**真实 mojian 二进制**，不触达真实 claude。
- [ ] 首次 `mojian run --path <proj>`：退出码 0，跑到 `brief` 关卡即停；`<MOJIAN_HOME>/logs/{project_id}/generation.jsonl` 新增一行（含 step/agent/token/cost 字段）。
- [ ] `mojian status --path <proj>`：stdout 含 project、phase，且含「卡在 brief 关卡 / 等待判定」类卡点提示（REQ-008）。
- [ ] `mojian decide brief REVISE --comment "钩子太弱"`：退出码 0；`<MOJIAN_HOME>/logs/{project_id}/decision.jsonl` 新增一行（gate=brief、verdict=REVISE、comment 含「钩子太弱」）。
- [ ] REQ-011 回喂：`decide brief REVISE --comment "钩子太弱"` 后再次 `mojian run --path <proj>`，新写入的 `generation.jsonl` 行的 inputs 中含上一轮评论「钩子太弱」文本。
- [ ] `mojian decide brief CONFIRMED` 后再次 `mojian run --path <proj>`：退出码 0，状态机能继续推进到下一动作/下一关卡（不再卡在 brief）——即 `run → decide → run` 通路成立（REQ-012）。
- [ ] 错误路径：当前不在某关卡时 `mojian decide brief CONFIRMED` 若 gate 不匹配，返回非 0 退出码 + 关卡不匹配错误信息（不 panic）。

## Dependencies

- 前置任务：TASK-005

## Log

- 2026-07-07 [planning-agent] created status=planned（依赖 TASK-005）
- 2026-07-08 [builder-agent] status ready → in_progress：认领任务
- 2026-07-08 [builder-agent] status in_progress → reviewing：run/decide/status 从桩转真逻辑——run 循环 next_action（Advance 顺推占位 phase / Generate 装配调 ClaudeCliRunner 写 generation.jsonl 置 brief 关卡即停 / HumanGate 停 / Idle 退出）、decide 用 clap 解析 gate/verdict/target/--comment/--file 并校验 pending_gate 匹配后 append_decision+apply_decision、status 追加卡点提示；cli.rs 桩用例替换为 MOJIAN_CLAUDE_CMD 假命令驱动的 run→decide→run 端到端。Build Verification（cargo check/build --workspace 0 error/warning）与 Builder Exit Criteria 6/6 通过；cargo test --workspace 全绿（CLI 5 passed，含新 run_decide_run_end_to_end；core 无回归）；手动 E2E 复核 REQ-008/011/012 + 关卡不匹配非 0 不 panic。
