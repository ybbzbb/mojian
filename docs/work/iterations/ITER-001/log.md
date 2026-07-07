# Build Log — ITER-001

## TASK-001 — 2026-07-07

变更文件：
- Cargo.toml（新增，workspace 根：[workspace] members + [workspace.dependencies]）
- .gitignore（新增，忽略 /target）
- crates/mojian-core/Cargo.toml（新增，依赖 thiserror/directories，均 workspace = true）
- crates/mojian-core/src/lib.rs（新增，门面 re-export CoreError）
- crates/mojian-core/src/error.rs（新增，CoreError/thiserror）
- crates/mojian-core/src/paths.rs（新增，数据目录分层解析 + 路径助手）
- crates/mojian-core/tests/paths.rs（新增，MOJIAN_HOME 解析集成测试）
- crates/mojian-cli/Cargo.toml（新增，[[bin]] name = "mojian"）
- crates/mojian-cli/src/main.rs（新增，最小可编译骨架）
- Cargo.lock（构建生成）

实现摘要：奠定 Cargo workspace 地基——根 Cargo.toml 用 [workspace.dependencies] 集中声明全部选型依赖版本（rusqlite bundled / clap derive / directories / serde derive / toml / blake3 / uuid v4 / time formatting / anyhow / thiserror / include_dir）；mojian-core 落地 CoreError（thiserror）与客户端数据目录分层解析 paths.rs（MOJIAN_HOME 环境变量 → directories::ProjectDirs 平台标准目录 → ~/.mojian/ 兜底），并提供 central_db_path()/spec_master_dir()/logs_dir() 路径助手；mojian-cli 产出二进制名 mojian 的最小骨架（完整 clap 分发在 TASK-006）。

Build Verification：
- `cargo check --workspace` → Finished, 0 error
- `cargo build --workspace` → Finished, 0 error；target/debug/mojian 已产出
- `cargo test -p mojian-core` → 1 passed, 0 failed（tests/paths.rs）
- `MOJIAN_HOME=$(mktemp -d) target/debug/mojian` → exit 0（不 panic）

Builder Exit Criteria：5/5 通过

已知风险：本机初始未安装 Rust 工具链，已安装 Rust stable 1.96.1（rustup minimal profile）以执行 Build Verification；后续迭代复用该工具链。依赖首次解析将 directories 锁定 v5、thiserror 锁定 v1（对齐 tech-design 选型版本）。

## TASK-002 — 2026-07-07

变更文件：
- crates/mojian-core/src/domain/mod.rs（新增，模块登记 + re-export 三枚举）
- crates/mojian-core/src/domain/sop_phase.rs（新增，SopPhase 10 变体 + as_db_str/TryFrom + 单测）
- crates/mojian-core/src/domain/chapter_state.rs（新增，ChapterState 7 态 + as_db_str/TryFrom + 单测）
- crates/mojian-core/src/domain/extract_status.rs（新增，ExtractStatus 3 态 + as_db_str/TryFrom + 单测）
- crates/mojian-core/src/lib.rs（修改，追加 `pub mod domain;` 与 re-export）
- crates/mojian-core/src/error.rs（修改，追加 `CoreError::UnknownDomainValue { kind, value }`）

实现摘要：在 mojian-core 落地两级领域状态机类型。SopPhase（Level 1 覆盖 SOP①②③ 全部 10 粗阶段）、ChapterState（SOP③ 章节七态 + 作废态 Void）、ExtractStatus（SOP① 抽取游标三态）三枚举变体逐字对齐迭代 tech-design.md 映射表与 docs/naming.md；每枚举以 as_db_str() + TryFrom<&str> 作为 DB 文本 ↔ 枚举的单一事实源，未知字符串返回 CoreError::UnknownDomainValue。本任务只落类型与文本互转，不实现状态转移逻辑。

范围说明：Builder Exit 要求 TryFrom「未知字符串返回 CoreError」，而 CoreError（error.rs，未列入 Allowed Files）无可用变体，故最小新增一个 UnknownDomainValue 变体（仅追加、不改既有变体）以满足验收口，已在任务 Log 显式记录。

Build Verification：
- `cargo check -p mojian-core` → Finished, 0 error
- `cargo test -p mojian-core domain` → 9 passed, 0 failed（三枚举各：as_db_str 表断言 / 全变体 roundtrip / 非法输入 Err）
- `cargo build --workspace` → exit 0
- `cargo test -p mojian-core` → 全通过（含既有 paths 测试）

Builder Exit Criteria：6/6 通过

已知风险：无。
