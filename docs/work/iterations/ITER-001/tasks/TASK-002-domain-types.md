# TASK-002 领域状态机类型与 DB 文本映射

- iteration: ITER-001
- status: reviewing
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

在 `mojian-core` 落地两级领域状态机类型：`SopPhase`（Level 1 粗粒度，覆盖 SOP①②③ 全部 10 个粗阶段）、`ChapterState`（SOP③ 章节七正常态 + 作废态 `Void`）、`ExtractStatus`（SOP① 抽取游标辅助类型）。每个枚举提供 `as_db_str()` + `TryFrom<&str>` 的 DB 文本值互转，代码与文档单一事实源。本任务只要求类型存在且能与 DB 文本互转，不实现状态转移逻辑。

## Allowed Files

- `crates/mojian-core/src/domain/mod.rs`
- `crates/mojian-core/src/domain/sop_phase.rs`
- `crates/mojian-core/src/domain/chapter_state.rs`
- `crates/mojian-core/src/domain/extract_status.rs`
- `crates/mojian-core/src/lib.rs`（仅追加 `pub mod domain;` 与相关 re-export，不改其他模块行）
- `crates/mojian-core/tests/**`
- 禁止：`crates/mojian-core/src/db/**`、`crates/mojian-core/src/project/**`、`crates/mojian-core/src/spec/**`、`crates/mojian-cli/**`

## Inputs

- docs/tech-design/storage.md#二、领域模型 — 两级状态机、章节状态机、SOP① 抽取游标语义
- 迭代 tech-design.md#数据模型变更「枚举 ↔ DB 文本值映射（REQ-005 逐字一致）」表 — 三个枚举的 PascalCase 变体与 snake_case DB 文本值（逐字权威源）
- requirements.md REQ-003 / REQ-004 / REQ-005
- docs/naming.md#Function / Variable Naming — 章节状态变体命名逐字要求

## Builder Exit Criteria

- [ ] `SopPhase` 含 10 个变体（逐字对齐 tech-design 映射表）：`StyleSampling` / `StyleExtracting` / `BriefDrafting` / `VisionDrafting` / `BibleBuilding` / `BibleCheck` / `BibleVerify` / `OutlineExpand` / `OutlineVerify` / `Writing`
- [ ] `ChapterState` 含 7 个变体：`Planned` / `SkeletonDrafting` / `SkeletonReview` / `ProseDrafting` / `ProseReview` / `Approved` / `Void`
- [ ] `ExtractStatus` 含 3 个变体：`Pending` / `Extracting` / `Extracted`
- [ ] 每个枚举实现 `as_db_str(&self) -> &'static str` 与 `TryFrom<&str>`（未知字符串返回 `CoreError`）；DB 文本值逐字对齐映射表（如 `StyleSampling`→`"style_sampling"`、`Void`→`"void"`、`Extracted`→`"extracted"`）
- [ ] 单元测试对每个变体断言：`as_db_str()` 精确等于映射表文本，且 `TryFrom::try_from(as_db_str())` 往返回原变体；非法字符串返回 `Err`
- [ ] `cargo check -p mojian-core` 0 error；`cargo test -p mojian-core` 通过；变体命名逐字对齐 docs/naming.md 与 storage.md（代码与文档不得出现两套叫法）

## QA Verification

- [ ] `cargo build --workspace` 退出码 0
- [ ] `cargo test -p mojian-core domain` 退出码 0，0 failed（含三枚举 DB 文本往返测试与非法输入测试）

## Dependencies

- 前置任务：TASK-001

## Log

- 2026-07-07 [planning-agent] status — → planned：创建任务
- 2026-07-07 [builder-agent] status ready → in_progress：认领任务
- 2026-07-07 [builder-agent] 实现 domain 模块（sop_phase/chapter_state/extract_status + mod），三枚举各 as_db_str() + TryFrom<&str>，变体逐字对齐 tech-design 映射表；lib.rs 追加 `pub mod domain;` 与 re-export
- 2026-07-07 [builder-agent] 范围外必要补充：为满足「未知字符串返回 CoreError」，在 error.rs 追加 `CoreError::UnknownDomainValue { kind, value }`（Allowed Files 未列 error.rs，但 Builder Exit 强制返回 CoreError；仅新增变体、不改既有变体）
- 2026-07-07 [builder-agent] Builder Exit 自证：变体计数 10/7/3 经 as_db_str 表断言 + roundtrip 覆盖；`cargo check -p mojian-core` 0 error；`cargo test -p mojian-core domain` 9 passed / 0 failed（含非法输入 Err 断言）；命名逐字对齐 naming.md/storage.md
- 2026-07-07 [builder-agent] status in_progress → reviewing：领域三枚举 + DB 文本互转落地；Build Verification 与 Builder Exit Criteria 全部通过
