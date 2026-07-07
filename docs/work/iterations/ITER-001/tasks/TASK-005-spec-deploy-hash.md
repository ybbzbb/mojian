# TASK-005 SPEC 主副本、部署与 hash 覆盖

- iteration: ITER-001
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

在 `mojian-core` 落地 SPEC 主副本管理与部署机制（占位骨架，仅验证「部署 + hash 覆盖」通路）：主副本 bootstrap（嵌入骨架首次落地到 `<data_dir>/spec/`）、权威 version/hash 读取、部署载荷树递归拷贝进项目、blake3 tree hash 计算、以及打开时的 hash 比对覆盖重部署。同时提供占位 SPEC 主副本资源树（`crates/mojian-cli/assets/spec/`）。真实提示词内容不在本迭代范围。

## Allowed Files

- `crates/mojian-core/src/spec/mod.rs`
- `crates/mojian-core/src/spec/master.rs`
- `crates/mojian-core/src/spec/deploy.rs`
- `crates/mojian-core/src/spec/hash.rs`
- `crates/mojian-core/src/lib.rs`（仅追加 `pub mod spec;` 与相关 re-export）
- `crates/mojian-core/Cargo.toml`（仅追加本模块所需依赖行：`blake3` / `include_dir`，均 workspace = true）
- `crates/mojian-cli/assets/spec/**`（占位 SPEC 主副本资源树，静态内容文件）
- `crates/mojian-core/tests/**`
- 禁止：`crates/mojian-core/src/domain/**`、`crates/mojian-core/src/db/**`、`crates/mojian-core/src/project/**`、`crates/mojian-cli/src/**`

## Inputs

- docs/tech-design/engine.md#项目文件布局 + #启动执行流 — 部署目标（`.claude/agents` / `.claude/skills` / `CLAUDE.md` / `prompts`）与 hash 覆盖（选项 A）
- 迭代 tech-design.md#SPEC 部署 + hash 覆盖机制（占位骨架）— 主副本落地 / 部署载荷 / 权威 version-hash / tree hash 定义 / 覆盖策略；#选型 5（blake3 tree hash）、#涉及模块的 `assets/spec/` 布局
- requirements.md REQ-011 / REQ-012 / REQ-013 / REQ-014 + 约束「项目内 SPEC 为纯可弃缓存（选项 A）」「项目目录内不得存放机器状态」

## Builder Exit Criteria

- [ ] `crates/mojian-cli/assets/spec/` 占位主副本树就位：`spec.toml`（`version = "0.0.1-skeleton"`，meta，不属部署载荷）、`CLAUDE.md`、`.claude/agents/.gitkeep`、`.claude/skills/.gitkeep`、`prompts/sop-1-style/README.md`、`prompts/sop-2-bible/README.md`、`prompts/sop-3-writing/README.md`
- [ ] `hash.rs::tree_hash(dir) -> Result<String, CoreError>`：对目录内每个文件按「相对路径升序」拼接「相对路径 + 文件内容 blake3」再整体 blake3，得确定性 hex tree hash（顺序无关、内容敏感）
- [ ] `master.rs`：bootstrap（`<data_dir>/spec/` 缺失时把嵌入骨架 `&include_dir::Dir` 写出，形成权威主副本）；`authoritative_version()` 读主副本 `spec.toml` 的 `version`；`authoritative_hash()` = 对部署载荷树（主副本除 `spec.toml` 外整棵树）算 `tree_hash`
- [ ] `deploy.rs`：`deploy_spec(payload_src, project_dir) -> Result<(String /*version*/, String /*hash*/), CoreError>` 先删除项目内旧部署目标条目再递归拷入载荷树（`.claude/agents` / `.claude/skills` / `CLAUDE.md` / `prompts`），返回 version + hash；`sync_if_drifted(project_dir, authoritative_hash, payload_src)` 实时重算**项目实际部署树** hash 与权威比对，不一致则覆盖重部署并返回 `(overwritten: true, new_hash)`，一致则不写返回 `false`
- [ ] 集成测试（临时目录）：`deploy_spec` 后项目内存在全部部署目标文件且不含 `spec.toml`；`tree_hash` 对相同内容不同创建顺序得同值、改一个文件内容得不同值；篡改项目内某部署文件后 `sync_if_drifted` 返回 overwritten 且文件被还原；未篡改时返回 not overwritten
- [ ] `cargo check -p mojian-core` 0 error；`cargo test -p mojian-core` 通过；命名遵循 docs/naming.md

## QA Verification

- [ ] `cargo build --workspace` 退出码 0
- [ ] `cargo test -p mojian-core spec` 退出码 0，0 failed（含 tree_hash 确定性/内容敏感、deploy 生成部署目标、drift 覆盖还原、无 drift 不写四类断言）

## Dependencies

- 前置任务：TASK-001

## Log

- 2026-07-07 [planning-agent] status — → planned：创建任务
