# Plan — ITER-001

updated: 2026-07-07

## Goal

搭起 mojian 执行器的最小运行地基：建立 Cargo workspace（`mojian-core` 库 + `mojian-cli` 二进制 `mojian`）、落地两级领域状态机类型与客户端中央 SQLite（12 表 + `schema_meta` 迁移）、实现 `mojian new`（建项目 + 中央 DB 登记 + SPEC 部署 + 写 `mojian.toml`）与 `mojian status`（读回 SOP phase），`run`/`decide` 留桩，达成「运行环境就位」里程碑。

## Tasks

| Task | Title | Type | Status | Owner | Depends On |
|------|-------|------|--------|-------|------------|
| TASK-001 | Cargo workspace 骨架 + 依赖基线 + 数据目录解析 | backend | ready | builder-agent | — |
| TASK-002 | 领域状态机类型与 DB 文本映射 | backend | planned | builder-agent | TASK-001 |
| TASK-003 | 中央 DB 建库与 schema_meta 迁移 | backend | planned | builder-agent | TASK-001 |
| TASK-004 | 项目登记与 mojian.toml manifest | backend | planned | builder-agent | TASK-002, TASK-003 |
| TASK-005 | SPEC 主副本、部署与 hash 覆盖 | backend | planned | builder-agent | TASK-001 |
| TASK-006 | CLI 命令 new / status + run / decide 桩 | backend | planned | builder-agent | TASK-004, TASK-005 |

## 需求覆盖

| REQ | Tasks |
|-----|-------|
| REQ-001（workspace 两 crate） | TASK-001 |
| REQ-002（cargo check + build --workspace 通过） | TASK-001（奠基）；每个任务 Builder Exit 复核 |
| REQ-003（SopPhase 两级阶段） | TASK-002 |
| REQ-004（ChapterState 七态 + Void） | TASK-002 |
| REQ-005（枚举命名逐字对齐 naming.md） | TASK-002 |
| REQ-006（12 表建库） | TASK-003 |
| REQ-007（schema_meta 迁移能力） | TASK-003 |
| REQ-008（mojian new：建目录 + DB 登记 + mojian.toml） | TASK-004（库能力）+ TASK-006（CLI 编排） |
| REQ-009（mojian status 读回 SOP phase） | TASK-004（load_project_state）+ TASK-006（CLI） |
| REQ-010（run/decide 桩） | TASK-006 |
| REQ-011（SPEC 主副本 + version/hash） | TASK-005 |
| REQ-012（new 部署 SPEC 进项目） | TASK-005（deploy 能力）+ TASK-006（new 调用） |
| REQ-013（打开时 hash 覆盖） | TASK-005（比对/覆盖能力）+ TASK-006（status 触发） |
| REQ-014（project 行 spec_version/hash 一致） | TASK-005（提供值）+ TASK-006（写回 DB） |

未覆盖需求：无。REQ-001~014 全部覆盖。

## Notes

- **执行顺序**：TASK-001 无依赖先行；完成后 TASK-002 / TASK-003 / TASK-005 均只依赖 TASK-001，可先后由 driver 提为 ready（单迭代单 builder 顺序执行，无并发写冲突）；TASK-004 依赖 TASK-002 + TASK-003；TASK-006 依赖 TASK-004 + TASK-005，是端到端收口点。
- **构建验证口**：`docs/devops.md` 的 Build Verification 当前为空，本迭代按 tech-design「DevOps 影响」节以 `cargo check` + `cargo build --workspace` 作为构建验证口（REQ-002）。`bundled` rusqlite 需系统 C 编译器。
- **测试隔离**：所有涉及数据目录 / DB / 部署的验证一律通过 `MOJIAN_HOME` 环境变量指向临时目录，禁止触碰真实 `~`（见 tech-design 选型 3 / 测试健壮性）。
- **共享文件说明**：`crates/mojian-core/src/lib.rs`（模块登记）、`crates/mojian-core/Cargo.toml` / `crates/mojian-cli/Cargo.toml`（依赖追加）在多个任务的 Allowed Files 中出现，各任务只允许改动本模块相关行（各任务已注明范围）。单迭代顺序执行，无并发写冲突。
- **纯 CLI 工具，无前端、无网络 API、无常驻服务、无 token 面**；QA Verification 全部以 `mojian` 二进制 / `cargo test` / `sqlite3` / 文件检查完成，无浏览器、无 UI 操作。
- 领域类型本迭代只要求「存在 + 与 DB 文本互转」，不实现状态转移函数（属 ITER-002+）。
