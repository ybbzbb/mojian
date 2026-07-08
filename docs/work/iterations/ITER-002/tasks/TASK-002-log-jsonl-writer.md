# TASK-002 log 模块：generation/decision JSONL 追加写 + 评论回读

- iteration: ITER-002
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

新增 `mojian-core::log` 模块，实现 generation / decision 两条 JSONL 日志的追加写与 decision 评论回读，落客户端 `<data_dir>/logs/{project_id}/`（`logs_dir()` 已就绪，本任务加写入器）。日志为 JSONL：一行一事件、只增不改、程序追加写（REQ-006 / REQ-010 / REQ-011 的日志基座）。产出为纯文件 IO + serde 序列化的 core 库 API，供后续 engine / context / CLI 调用。

## Allowed Files

- crates/mojian-core/src/log/mod.rs（新增）
- crates/mojian-core/src/lib.rs（仅追加 `pub mod log;` 与 log 公共项 re-export）
- crates/mojian-core/tests/log_jsonl.rs（新增集成测试）
- 禁止：crates/mojian-core/src/{sdk,context,engine,state}/**（本任务不建这些模块）
- 禁止：crates/mojian-cli/**
- 禁止：crates/mojian-core/src/paths.rs（`logs_dir()` 已就绪，只读不改）
- 禁止：.esr_harnass/**、.claude/**

## Inputs

- docs/work/iterations/ITER-002/tech-design.md#数据模型变更 — generation.jsonl / decision.jsonl 字段清单；check.jsonl 本迭代不写
- docs/work/iterations/ITER-002/tech-design.md#API 变更 — `log::append_generation` / `append_decision` / `read_decision_comments` 签名
- crates/mojian-core/src/paths.rs — `logs_dir()` 返回 `<data_dir>/logs`，事件文件置于其下 `{project_id}/`

## Builder Exit Criteria

- [ ] 新增 `GenerationEvent` 结构体（`serde::Serialize`），字段至少含：step、agent、spec_path、spec_hash、inputs（切片列表，每项含 path/anchor + content_hash）、token_in、token_out、cost、ts（对齐 tech-design.md 数据模型表 generation.jsonl 行）。
- [ ] 新增 `DecisionEvent` 结构体（`Serialize` + `Deserialize`），字段至少含：gate、verdict、target、comment、ts（对齐 decision.jsonl 行）。
- [ ] `append_generation(project_id, &GenerationEvent) -> Result<(), CoreError>`：确保 `<data_dir>/logs/{project_id}/` 目录存在，以 `serde_json::to_string` 序列化为**单行**并**追加**写入 `generation.jsonl`（每行末尾换行；不覆盖已有内容）。
- [ ] `append_decision(project_id, &DecisionEvent) -> Result<(), CoreError>`：同上写 `decision.jsonl`。
- [ ] `read_decision_comments(project_id, gate, target) -> Result<Vec<String>, CoreError>`：读 `decision.jsonl`，逐行反序列化，筛出匹配 `gate`（且 target 匹配或 target 为空视为全局）且 `comment` 非空的评论，按写入顺序返回（REQ-011 回喂来源）；文件不存在返回空 Vec。
- [ ] 不写 `check.jsonl`（裁决①，本迭代排除）。
- [ ] `lib.rs` 导出上述类型与函数；`cargo check` 0 error。
- [ ] 单元测试（`#[cfg(test)]`）覆盖：序列化后为合法单行 JSON、字段 rename/形状正确、`read_decision_comments` 的 gate/target 过滤逻辑（含空 target 与不匹配 target 边界）。
- [ ] 命名遵循 docs/naming.md（函数 snake_case 动词开头、类型 PascalCase）。

## QA Verification

- [ ] `MOJIAN_HOME=<临时目录> cargo test -p mojian-core --test log_jsonl` 退出码 0（集成测试真实写盘到隔离 data_dir 并断言磁盘文件内容）。
- [ ] 集成测试须断言：连续两次 `append_generation` 后，`<MOJIAN_HOME>/logs/{project_id}/generation.jsonl` 实际为**两行**、每行可被 `serde_json` 反序列化、且第一行内容未被第二次写入改动（只增不改）。
- [ ] 集成测试须断言：写入两条不同 gate 的 `DecisionEvent`（各带 comment）后，`read_decision_comments(project_id, "brief", None)` 只返回 gate == "brief" 那条评论文本。

## Dependencies

- 前置任务：TASK-001

## Log

- 2026-07-07 [planning-agent] created status=planned（依赖 TASK-001）
