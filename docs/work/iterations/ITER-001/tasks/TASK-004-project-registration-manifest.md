# TASK-004 项目登记与 mojian.toml manifest

- iteration: ITER-001
- status: reviewing
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

在 `mojian-core` 落地项目登记与身份标记读写：`register_project`（在单事务内插入 `project` 行与 `project_state` 行，初始 `sop_phase = style_sampling`）、`load_project_state`（按 `project_id` 读回当前 SOP phase）、`update_project_spec`（写回 `project.spec_version` / `spec_hash`，供 CLI 满足 REQ-014），以及 `mojian.toml` 的读写（`ProjectManifest { project_id, spec_version }`）。

## Allowed Files

- `crates/mojian-core/src/project/mod.rs`
- `crates/mojian-core/src/project/manifest.rs`
- `crates/mojian-core/src/lib.rs`（仅追加 `pub mod project;` 与相关 re-export）
- `crates/mojian-core/Cargo.toml`（仅追加本模块所需依赖行：`uuid`(features `v4`) / `time`(features `formatting`) / `serde`(features `derive`) / `toml`，均 workspace = true）
- `crates/mojian-core/tests/**`
- 禁止：`crates/mojian-core/src/domain/**`、`crates/mojian-core/src/db/**`、`crates/mojian-core/src/spec/**`、`crates/mojian-cli/**`

## Inputs

- docs/tech-design/storage.md#五、DB 表设计 — `project` / `project_state` 列定义
- docs/tech-design/storage.md#四、项目（运行环境） — `mojian.toml` 身份标记语义（project_id + 已部署 SPEC 版本）
- 迭代 tech-design.md#API 变更「`mojian new`」步骤 2/3、#选型 4（serde+toml manifest）、#数据模型变更（`project_state.sop_phase` 初值）
- requirements.md REQ-008 / REQ-009 + 约束「新建项目初始 phase = style_sampling」

## Builder Exit Criteria

- [ ] `register_project(conn, name, path) -> Result<String, CoreError>`：生成 UUID v4 `project_id`，在**单事务内**插入 `project` 行（`name`、`path` 绝对路径、`created_at`/`updated_at` RFC3339）与 `project_state` 行（`sop_phase = SopPhase::StyleSampling.as_db_str()` 即 `"style_sampling"`），返回 `project_id`
- [ ] `load_project_state(conn, project_id) -> Result<SopPhase, CoreError>`（或返回 phase 文本）：按 `project_id` 读 `project_state.sop_phase` 并经 `SopPhase::TryFrom` 还原；无此项目返回 `CoreError`
- [ ] `update_project_spec(conn, project_id, spec_version, spec_hash)`：更新 `project` 行的 `spec_version` / `spec_hash` / `updated_at`
- [ ] `manifest.rs`：`ProjectManifest { project_id: String, spec_version: String }`（`#[derive(Serialize, Deserialize)]`）；`write_manifest(dir, &manifest)` 写 `<dir>/mojian.toml`；`read_manifest(dir) -> Result<ProjectManifest, CoreError>` 读回，缺文件返回 `CoreError`
- [ ] 集成测试：`open_central_db` 建临时库 → `register_project` → `load_project_state` 返回 `SopPhase::StyleSampling`；`write_manifest` 后 `read_manifest` 往返得到相同 `project_id` 与 `spec_version`
- [ ] `cargo check -p mojian-core` 0 error；`cargo test -p mojian-core` 通过；命名遵循 docs/naming.md

## QA Verification

- [ ] `cargo build --workspace` 退出码 0
- [ ] `cargo test -p mojian-core project` 退出码 0，0 failed（含 register→load 得 `style_sampling` 断言、mojian.toml 往返断言）

## Dependencies

- 前置任务：TASK-002, TASK-003

## Log

- 2026-07-07 [planning-agent] status — → planned：创建任务
- 2026-07-07 [builder-agent] status ready → in_progress：认领任务
- 2026-07-07 [builder-agent] status in_progress → reviewing：实现 project 模块（register_project 单事务登记 project+project_state 初值 style_sampling / load_project_state / update_project_spec / mojian.toml 读写），lib.rs 登记并 re-export，Cargo.toml 追加 uuid/time/serde/toml；Build Verification（cargo check -p mojian-core、cargo test -p mojian-core project 4 passed 0 failed、cargo build --workspace exit 0）与 Builder Exit Criteria 全部通过
