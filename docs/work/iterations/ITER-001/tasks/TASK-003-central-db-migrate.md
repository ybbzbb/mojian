# TASK-003 中央 DB 建库与 schema_meta 迁移

- iteration: ITER-001
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

在 `mojian-core` 落地客户端中央 SQLite（`central.db`）：把 storage.md「五」的逐字 12 表 DDL 做成 v1 建表，实现基于 `schema_meta.schema_version` 的自研迁移运行器（不用 PRAGMA `user_version`），并提供唯一 DB 入口 `open_central_db(path)`（打开即跑迁移）。

## Allowed Files

- `crates/mojian-core/src/db/mod.rs`
- `crates/mojian-core/src/db/schema.rs`
- `crates/mojian-core/src/db/migrate.rs`
- `crates/mojian-core/src/lib.rs`（仅追加 `pub mod db;` 与相关 re-export）
- `crates/mojian-core/Cargo.toml`（仅追加 `rusqlite`(workspace = true, features `bundled`) 依赖行）
- `crates/mojian-core/tests/**`
- 禁止：`crates/mojian-core/src/domain/**`、`crates/mojian-core/src/project/**`、`crates/mojian-core/src/spec/**`、`crates/mojian-cli/**`

## Inputs

- docs/tech-design/storage.md#五、DB 表设计 — 12 表逐字 DDL（`project` / `project_state` / `reference_book` / `volume` / `batch` / `chapter` / `artifact_ref` / `bible_version` / `void_record` / `stat` / `config` / `schema_meta`）
- 迭代 tech-design.md#数据模型变更「建库与迁移（REQ-006/007）」+ 选型 1（rusqlite bundled）
- requirements.md REQ-006 / REQ-007

## Builder Exit Criteria

- [ ] `schema.rs` 持有 storage.md「五」逐字 12 表 DDL，定义 `SCHEMA_VERSION: i64 = 1`；建表以 `execute_batch` 在单事务内完成全部 12 表
- [ ] `migrate.rs` 迁移运行器**键于 `schema_meta` 表**（非 `user_version`）：打开连接 `PRAGMA foreign_keys = ON`；`schema_meta` 无行（全新库）→ 建 12 表 + `INSERT INTO schema_meta(schema_version) VALUES (1)`；有行 → 读 `schema_version`，v1 无后续步骤（no-op）；整体在事务内，失败回滚
- [ ] `open_central_db(path) -> Result<Connection, CoreError>` 为唯一 DB 入口：打开连接 → 跑迁移 → 返回 `Connection`
- [ ] 集成测试（临时路径建库）断言：新建库后 `sqlite_master` 含全部 12 个具名表；`SELECT schema_version FROM schema_meta` 返回 `1`；对同一路径二次 `open_central_db` 幂等（不重复建表、不报错、schema_version 仍为 1）
- [ ] `cargo check -p mojian-core` 0 error；`cargo test -p mojian-core` 通过；命名遵循 docs/naming.md（列名 snake_case）

## QA Verification

- [ ] `cargo build --workspace` 退出码 0
- [ ] `cargo test -p mojian-core db` 退出码 0，0 failed（含 12 表建库断言、`schema_meta.schema_version==1` 断言、二次打开幂等断言）

## Dependencies

- 前置任务：TASK-001

## Log

- 2026-07-07 [planning-agent] status — → planned：创建任务
