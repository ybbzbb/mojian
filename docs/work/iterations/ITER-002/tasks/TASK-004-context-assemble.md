# TASK-004 context 模块：manifest + 符号解析 + 切片 + assemble_bundle + 占位 SPEC 步

- iteration: ITER-002
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

新增 `mojian-core::context` 模块，实现切片装配全链（IMPL-3 核心）：解析 TOML sidecar 输入契约 manifest（REQ-001）、按当前 DB 状态解析符号引用（REQ-001/002）、段级（`#anchor` 稳定小标题抽取）/ 整文件双粒度切片 + blake3 内容 hash（REQ-002）、把 `decision.jsonl` 相关人类评论回喂进 `inputs`（REQ-011），组装五字段 `Bundle`（REQ-003）。同时落地一个占位可跑步骤资产（`brief-agent.md` + `brief-agent.manifest.toml`），使 manifest 格式与其解析器 co-evolve，并为后续 CLI 端到端提供一个能在新建项目部署后即刻解析的 brief 通路。

## Allowed Files

- crates/mojian-core/src/context/manifest.rs（新增：TOML sidecar 模型 + 读取）
- crates/mojian-core/src/context/symbol.rs（新增：符号 `<source>.<selector>[:{params}][#anchor]` 解析 + 占位代入）
- crates/mojian-core/src/context/slice.rs（新增：段级 / 整文件切片 + blake3 hash）
- crates/mojian-core/src/context/assemble.rs（新增：assemble_bundle）
- crates/mojian-core/src/context/mod.rs（新增：子模块聚合 + 导出）
- crates/mojian-core/src/lib.rs（仅追加 `pub mod context;` 与 context 公共项 re-export）
- crates/mojian-core/src/error.rs（如需补 manifest 非法 / 符号无法解析变体细节；主体已在 TASK-001 预置）
- crates/mojian-cli/assets/spec/.claude/agents/brief-agent.md（新增：占位 agent 提示词）
- crates/mojian-cli/assets/spec/.claude/agents/brief-agent.manifest.toml（新增：占位步骤输入契约）
- crates/mojian-core/tests/context_assemble.rs（新增集成测试）
- 禁止：crates/mojian-core/src/{sdk,log,engine,state}/**（复用其公共 API，不改其实现）
- 禁止：crates/mojian-cli/src/**（本任务不动 CLI 代码）
- 禁止：crates/mojian-core/src/spec/deploy.rs（部署目标已含 .claude/agents，无需改）
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#选型对比 — 选型 3-A（TOML sidecar manifest）、选型 5-A（手写符号解析器）
- docs/work/iterations/ITER-002/tech-design.md#API 变更 — `context::assemble_bundle(conn, project_id, project_dir, manifest_path) -> Result<Bundle, CoreError>`
- docs/tech-design/engine.md#切片装配器 + 输入契约 manifest — 符号引用语义、`write:` → write_scope、`#anchor` 段级切片、参数优先/缺失回退读文件
- crates/mojian-core/src/sdk/mod.rs — `Bundle` 五字段类型（TASK-003 产出）
- crates/mojian-core/src/log/mod.rs — `read_decision_comments`（TASK-002 产出，REQ-011 回喂来源）
- crates/mojian-cli/assets/spec/spec.toml + prompts/ — 占位 manifest 的 inputs 可指向部署后必然存在的文件

## Builder Exit Criteria

- [ ] `manifest.rs`：TOML sidecar 模型（`agent` / `inputs`（符号引用数组）/ `write`（白名单）/ `output_contract`），用既有 `toml` + `serde` 反序列化；非法 manifest 返回 manifest 非法错误变体。
- [ ] `symbol.rs`：手写解析器切分 `<source>.<selector>[:{params}][#anchor]`（如 `bible.style#skeleton`、`plan.chapters:{batch}`、`prev_skeleton:{ch-1}`），并按传入的当前状态代入 `{arc_id}` / `{batch}` / `{ch-1}` 等占位；无法解析的符号返回符号无法解析错误变体。
- [ ] `slice.rs`：两种粒度——整文件切片（读整文件）与段级切片（依 `#anchor` 稳定小标题锚点抽取命名段落）；两者均计算内容 blake3 hash（复用既有 `blake3` 依赖）；被引用文件缺失返回明确 `CoreError`。
- [ ] `assemble.rs`：`assemble_bundle(conn, project_id, project_dir, manifest_path)` 串起「读 manifest → 解析符号 → 切片 → 调 `log::read_decision_comments` 回喂人类评论进 `inputs` → 由 `write:` 推导 `write_scope` → 组五字段 `Bundle`」（REQ-003/011）。
- [ ] `brief-agent.manifest.toml`：最小但真实的输入契约，`inputs` 至少含**整文件切片一项 + `#anchor` 段级切片一项**、`write` 白名单至少一项；其 `inputs` 引用的文件在 `mojian new` 部署后于项目内**必然存在**（指向部署产物，使 assemble 无需手工种子即可解析）；`brief-agent.md` 为占位提示词。
- [ ] `lib.rs` 导出 `assemble_bundle` 及必要公共类型；`cargo check` 0 error。
- [ ] 单元测试（`#[cfg(test)]`）覆盖：符号文法四类切分（含 `#anchor`、`:{params}`、纯整文件）、占位代入、段级 `#anchor` 抽取正确边界（锚点命中/未命中）、整文件切片 hash 稳定。
- [ ] 命名遵循 docs/naming.md（`assemble_bundle` 等函数 snake_case 动词开头）。

## QA Verification

- [ ] `cargo test -p mojian-core --test context_assemble` 退出码 0（集成测试在隔离 `MOJIAN_HOME` + 临时项目目录中部署占位 SPEC、种子最小 DB 行，真实调用 `assemble_bundle` 解析磁盘上的 `brief-agent.manifest.toml`）。
- [ ] 集成测试须断言：`assemble_bundle` 返回的 `Bundle.agent` 指向部署的 brief-agent、`write_scope` 由 manifest `write:` 推导（非空、与白名单一致）、`inputs` 含被切片文件的 content_hash。
- [ ] 集成测试须断言：预先向 `decision.jsonl` 写入一条 gate == "brief" 且带 comment 的记录后，`assemble_bundle` 组出的 `Bundle.inputs` 中含该评论文本（REQ-011 回喂通路）。

## Dependencies

- 前置任务：TASK-002, TASK-003

## Log

- 2026-07-07 [planning-agent] created status=planned（依赖 TASK-002 log、TASK-003 sdk）
