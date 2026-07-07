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
