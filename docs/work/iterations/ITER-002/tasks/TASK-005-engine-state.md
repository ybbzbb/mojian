# TASK-005 engine + state 模块：next_action / apply_* + 运行时 DB 行读写

- iteration: ITER-002
- status: done
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-08

## Goal

新增 `mojian-core::engine`（状态机核心）与 `mojian-core::state`（运行时 DB 行读写），把「装配 → 调 SDK → 落库/推进」的机制固化为纯函数状态机（守 AP-001）。`engine::next_action` 计算下一个动作、`apply_generation` / `apply_decision` 收敛落库；`state` 承载 `project_state` 推进 / 关卡标记（`cursors` JSON `pending_gate`）/ chapter 状态 / void_record / artifact_ref 的读写。落地裁决①（客观检查步留位空过）、裁决③（VOID 最小语义）。

## Allowed Files

- crates/mojian-core/src/engine/mod.rs（新增：Action / Verdict 枚举、next_action、apply_generation、apply_decision）
- crates/mojian-core/src/state/mod.rs（新增：运行时 DB 行读写 helper）
- crates/mojian-core/src/lib.rs（仅追加 `pub mod engine; pub mod state;` 与公共项 re-export）
- crates/mojian-core/src/error.rs（如需补关卡状态不匹配变体细节；主体已在 TASK-001 预置）
- crates/mojian-core/tests/engine_loop.rs（新增集成测试）
- 禁止：crates/mojian-core/src/{sdk,log,context}/**（复用其公共 API）
- 禁止：crates/mojian-core/src/db/schema.rs（无 schema 迁移，SCHEMA_VERSION 保持 1，不加列）
- 禁止：crates/mojian-cli/**
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#选型对比 — 选型 6-A（run/decide 逻辑落 core::engine，纯函数 + apply_* 收敛 DB 写）
- docs/work/iterations/ITER-002/tech-design.md#API 变更 — `engine::next_action / apply_generation / apply_decision / Verdict`；`state::{advance_sop_phase, set_gate, clear_gate, load_chapter, update_chapter_status, insert_void_record, upsert_artifact_ref}`
- docs/work/iterations/ITER-002/tech-design.md#数据模型变更 — 落库映射表（sop_phase / cursors.pending_gate / chapter.status / void_record / artifact_ref）；无迁移
- docs/work/iterations/ITER-002/tech-design.md#依赖与风险 — 占位简化（style_sampling/style_extracting no-op 直进；仅 brief_drafting 完整 Generate + brief 完整关卡）；接缝（Generate 与关卡间预留客观检查步位，本迭代空过）
- crates/mojian-core/src/domain/sop_phase.rs — 既有 SopPhase 枚举与 as_db_str 映射
- crates/mojian-core/src/db/schema.rs — chapter / void_record / artifact_ref / project_state 列（只读参考）

## Builder Exit Criteria

- [ ] `engine::Action` 枚举至少含：`Advance`、`Generate { agent, manifest }`、`HumanGate { gate }`、`Idle`（对齐 tech-design.md「API 变更 / mojian run」行为分支）。
- [ ] `engine::Verdict` 枚举含 `CONFIRMED` / `REVISE` / `VOID`（对齐 REQ-009）。
- [ ] `next_action(&RunState) -> Action` 为**纯函数**：SOP① `style_sampling` / `style_extracting` 映射为 `Advance`（占位 no-op 直进）；`brief_drafting` 映射为 `Generate`；`cursors.pending_gate` 存在时映射为 `HumanGate`；无可跑动作映射为 `Idle`。Generate 与关卡之间**预留客观检查步位**但本迭代空过（不写 check.jsonl；裁决①）。
- [ ] `apply_generation(...)`：置 `brief` 关卡标记（`state::set_gate` 写 `cursors.pending_gate`）+ 写 `artifact_ref`（切片 content_hash，与 generation.jsonl 同源）。
- [ ] `apply_decision(...)` 按 `Verdict`：`CONFIRMED` → `clear_gate` + 推进（`brief` → `vision_drafting`）；`REVISE` → `clear_gate` + 回退对应细粒度状态（brief 关卡回 `brief_drafting`；章节关卡回 `skeleton_drafting`）；`VOID <CH>` → `insert_void_record` + `update_chapter_status(void → planned)`（裁决③ 最小语义，**不**做圣经级联 / 过期检测）。
- [ ] `state` 模块实现：`advance_sop_phase` / `set_gate` / `clear_gate`（读改写 `project_state.cursors` JSON，用 serde_json）/ `load_chapter` / `update_chapter_status` / `insert_void_record` / `upsert_artifact_ref`，全部走 rusqlite，无 schema 迁移。
- [ ] `lib.rs` 导出上述公共项；`cargo check` 0 error。
- [ ] 单元测试（`#[cfg(test)]`）覆盖 `next_action` 每条 phase→action 映射（含 pending_gate 存在时优先出 HumanGate、Idle 出口）。
- [ ] 命名遵循 docs/naming.md（phase 名与 SopPhase 逐字对齐，函数 snake_case 动词开头）。

## QA Verification

- [ ] `MOJIAN_HOME=<临时目录> cargo test -p mojian-core --test engine_loop` 退出码 0（集成测试在隔离临时 DB 中登记项目、种子 volume/batch/chapter 行，真实调用 `apply_*` 并回读 DB 断言）。
- [ ] 集成测试须断言：`apply_generation` 后回读 `project_state.cursors` 含 `pending_gate == "brief"`；`apply_decision(CONFIRMED, brief)` 后 `pending_gate` 被清除且 `sop_phase == "vision_drafting"`。
- [ ] 集成测试须断言：种子一个 `status == "void"` 的 chapter，`apply_decision(VOID, CH-xxx)` 后 `void_record` 新增一行且该 chapter `status == "planned"`（裁决③ 最小语义）。

## Dependencies

- 前置任务：TASK-002, TASK-003, TASK-004

## Log

- 2026-07-07 [planning-agent] created status=planned（依赖 TASK-002/003/004）
- 2026-07-08 [builder-agent] status ready → in_progress：认领任务
- 2026-07-08 [builder-agent] status in_progress → reviewing：实现 engine（Action/Verdict/next_action 纯函数 + apply_generation/apply_decision）与 state（7 个运行时 DB 读写 helper + read_pending_gate），lib.rs 导出，error.rs 补 MissingDecisionTarget 变体；Build Verification 与 Builder Exit Criteria 全部通过。
  - Exit①②（Action/Verdict 枚举）：engine/mod.rs 定义 `Action{Advance,Generate{agent,manifest},HumanGate{gate},Idle}` 与 `Verdict{Confirmed,Revise,Void}`（含 as_db_str / TryFrom<&str>）；`verdict_parses_and_round_trips` 单测证。
  - Exit③（next_action 纯函数）：`RunState`（sop_phase + pending_gate）→ Action 无 IO 映射；pending_gate 优先 HumanGate（Generate 与关卡间检查步位空过，不写 check.jsonl）；style_sampling/style_extracting→Advance、brief_drafting→Generate、其余→Idle。单测 `style_phases_map_to_advance` / `brief_drafting_maps_to_generate` / `pending_gate_takes_priority_over_phase` / `unwired_phase_maps_to_idle`。
  - Exit④（apply_generation）：为每个 InputSlice upsert_artifact_ref(kind=input) + set_gate(brief)；集成测试 `generation_sets_brief_gate_then_confirm_advances` 断言回读 cursors.pending_gate=="brief" 且 artifact_ref 2 行。
  - Exit⑤（apply_decision 三判定）：CONFIRMED→clear_gate+advance vision_drafting；REVISE→clear_gate+回退（brief→brief_drafting / 章节 target→skeleton_drafting）；VOID→insert_void_record+update_chapter_status(planned)，target 缺失报 MissingDecisionTarget。集成测试三例（confirm/revise/void）全绿。
  - Exit⑥（state 模块）：advance_sop_phase/set_gate/clear_gate/read_pending_gate（cursors JSON via serde_json，保留其他游标键）/load_chapter/update_chapter_status/insert_void_record/upsert_artifact_ref 全走 rusqlite，SCHEMA_VERSION 保持 1、无迁移。
  - Exit⑦（导出 + cargo check）：lib.rs 追加 `pub mod engine; pub mod state;` 与 re-export；`cargo check --workspace` 0 error。
  - Exit⑧（单元测试覆盖映射）：见 Exit③ 四个单测 + state 模块 6 个单测。
  - Exit⑨（命名）：phase 名逐字对齐 SopPhase，函数 snake_case 动词开头，Verdict 变体 PascalCase。
  - Build Verification：`cargo check -p mojian-core` 与 `cargo check --workspace` 0 error（未改 Cargo.toml，serde_json 既有依赖，无新依赖、无 schema 迁移，未触发打包校验条件）；`cargo test -p mojian-core --lib` 54 passed；`MOJIAN_HOME=<tmp> cargo test -p mojian-core --test engine_loop` 3 passed（EXIT=0）。
- 2026-07-08 [qa-agent] status reviewing → done：QA Verification 3/3 通过（cargo build --workspace EXIT=0；engine_loop 3 passed 真跑落库回读；apply_generation→pending_gate=="brief"+artifact_ref 2 行；apply_decision(CONFIRMED,brief)→pending_gate 清除+sop_phase=="vision_drafting"；VOID(CH-7)→void_record 新增 1 行+chapter status=="planned"；engine 5 单测 / state 10 单测 0 failed）
