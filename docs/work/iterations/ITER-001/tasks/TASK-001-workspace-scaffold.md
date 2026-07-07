# TASK-001 Cargo workspace 骨架 + 依赖基线 + 数据目录解析

- iteration: ITER-001
- status: reviewing
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

一次性奠定 workspace 地基：建立 Cargo workspace（成员 `mojian-core` 库 + `mojian-cli` 二进制 `mojian`）、在根 `Cargo.toml` 的 `[workspace.dependencies]` 集中声明全部选型依赖版本、落地 `CoreError`（thiserror）与客户端数据目录分层解析 `paths.rs`。产出：两 crate 均可编译（`cargo check`/`cargo build --workspace` 通过），后续所有模块挂载到这套骨架上。

## Allowed Files

- `Cargo.toml`（workspace 根）
- `crates/mojian-core/Cargo.toml`
- `crates/mojian-core/src/lib.rs`
- `crates/mojian-core/src/error.rs`
- `crates/mojian-core/src/paths.rs`
- `crates/mojian-cli/Cargo.toml`
- `crates/mojian-cli/src/main.rs`（本任务仅最小可编译骨架，完整 clap 分发在 TASK-006）
- `crates/mojian-core/tests/**`
- `.gitignore`
- 禁止：`crates/mojian-core/src/domain/**`、`crates/mojian-core/src/db/**`、`crates/mojian-core/src/project/**`、`crates/mojian-core/src/spec/**`、`crates/mojian-cli/src/commands/**`、`crates/mojian-cli/assets/**`

## Inputs

- docs/tech-design/storage.md#三、客户端（中央） — 数据目录布局（`central.db` / `spec/` / `logs/{project_id}/`）
- 迭代 tech-design.md 选型 3（directories 分层解析）+ 选型 6（uuid/time/anyhow/thiserror/include_dir）+「采用方案」+「涉及模块」workspace 布局 + `crates/mojian-core/src/paths.rs` / `error.rs` 行
- requirements.md 约束「客户端数据目录位置」（默认 `~/.mojian/`）+ REQ-001 / REQ-002

## Builder Exit Criteria

- [ ] `cargo check` 与 `cargo build --workspace` 均 0 error；`mojian-cli` 产出二进制名为 `mojian`（`[[bin]] name = "mojian"`）
- [ ] 根 `Cargo.toml` 含 `[workspace]`（members = 两 crate）与 `[workspace.dependencies]`，逐条声明全部选型版本：`rusqlite`(features `bundled`)、`clap`(feature `derive`)、`directories`、`serde`(feature `derive`)、`toml`、`blake3`、`uuid`(feature `v4`)、`time`(feature `formatting`)、`anyhow`、`thiserror`、`include_dir`（版本对齐 tech-design 选型节）
- [ ] `error.rs` 定义 `CoreError`（`thiserror::Error`），并被 `lib.rs` re-export
- [ ] `paths.rs` 实现数据目录分层解析：`MOJIAN_HOME` 环境变量 → 平台标准目录（`directories::ProjectDirs`，Linux XDG / macOS Application Support）→ 兜底 `~/.mojian/`；并提供 `central_db_path()` / `spec_master_dir()` / `logs_dir()` 等路径助手（返回 `<data_dir>` 下对应子路径）
- [ ] 单元/集成测试覆盖 `paths` 解析：设置 `MOJIAN_HOME` 时 `data_dir()` 精确返回该目录，`central_db_path()` == `<MOJIAN_HOME>/central.db`；`cargo test -p mojian-core` 通过
- [ ] 命名遵循 docs/naming.md（crate 目录 `mojian-` 前缀 kebab-case、模块 snake_case、类型 PascalCase）

## QA Verification

- [ ] `cargo build --workspace` 退出码 0，`target/debug/mojian` 文件存在
- [ ] `cargo test -p mojian-core` 退出码 0，0 failed（含 paths 解析测试）
- [ ] `MOJIAN_HOME=$(mktemp -d) target/debug/mojian` 退出码 0（最小骨架可运行，不 panic）
- [ ] `cargo check --workspace` 退出码 0，无 error 输出

## Dependencies

- 前置任务：无

## Log

- 2026-07-07 [planning-agent] status — → ready：创建任务
- 2026-07-07 [builder-agent] status ready → in_progress：认领任务
- 2026-07-07 [builder-agent] status in_progress → reviewing：建立 Cargo workspace（根 Cargo.toml 的 [workspace] + [workspace.dependencies] 统一版本）、mojian-core（CoreError/thiserror + paths.rs 分层解析 + 路径助手 + lib.rs 门面）、mojian-cli（二进制名 mojian 最小骨架）；Build Verification 与 Builder Exit Criteria 全部通过
- 2026-07-07 [builder-agent] Builder Exit Criteria 验证：
  - `cargo check --workspace` 与 `cargo build --workspace` 均 0 error；`target/debug/mojian` 已产出（`[[bin]] name = "mojian"`）
  - 根 Cargo.toml `[workspace.dependencies]` 逐条声明 rusqlite(bundled)/clap(derive)/directories/serde(derive)/toml/blake3/uuid(v4)/time(formatting)/anyhow/thiserror/include_dir
  - `CoreError`（`thiserror::Error`）定义于 error.rs，经 lib.rs `pub use error::CoreError` re-export
  - paths.rs 实现 MOJIAN_HOME → `directories::ProjectDirs` → `~/.mojian/` 兜底解析，并提供 central_db_path()/spec_master_dir()/logs_dir()
  - `cargo test -p mojian-core` 通过（集成测试 tests/paths.rs：设 MOJIAN_HOME 后 data_dir()==该目录、central_db_path()==<home>/central.db）
  - 命名遵循 docs/naming.md（crate mojian- 前缀 kebab-case、模块 snake_case、类型 PascalCase）
  - 环境说明：本机初始无 cargo，已安装 Rust stable 1.96.1（rustup minimal）以执行 Build Verification
