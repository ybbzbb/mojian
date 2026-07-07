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

## TASK-003 — 2026-07-07

变更文件：
- crates/mojian-core/src/db/schema.rs（新增，SCHEMA_VERSION=1 + V1_SCHEMA_SQL 逐字 12 表 DDL）
- crates/mojian-core/src/db/migrate.rs（新增，键于 schema_meta 的有序步骤迁移运行器）
- crates/mojian-core/src/db/mod.rs（新增，唯一 DB 入口 open_central_db）
- crates/mojian-core/src/lib.rs（修改，追加 `pub mod db;` 与 `pub use db::{open_central_db, SCHEMA_VERSION}`）
- crates/mojian-core/src/error.rs（修改，追加 `CoreError::Db(#[from] rusqlite::Error)`）
- crates/mojian-core/Cargo.toml（修改，追加 rusqlite 依赖 + [dev-dependencies] rusqlite）
- crates/mojian-core/tests/db.rs（新增，建库集成测试）

实现摘要：在 mojian-core 落地客户端中央 SQLite 建库与迁移。schema.rs 持 SCHEMA_VERSION: i64 = 1 与逐字对齐 storage.md「五」的 12 表 DDL（project / project_state / reference_book / volume / batch / chapter / artifact_ref / bible_version / void_record / stat / config / schema_meta）。migrate.rs 实现自研迁移运行器，键于 schema_meta.schema_version（非 PRAGMA user_version）：事务外先 `PRAGMA foreign_keys = ON`，事务内按有序编号步骤升级——全新库（无 schema_meta 表）从版本 0 跑 v1 建全 12 表并 `INSERT schema_meta(schema_version)=1`，已有库读版本、v1 无后续步骤（no-op），整体事务失败回滚。mod.rs 的 open_central_db(path) = 打开连接 → 跑迁移 → 返回 Connection，是所有 DB 访问的唯一入口。

范围说明：error.rs、Cargo.toml 的 [dev-dependencies] 属 Allowed Files 外/附加改动——error.rs 仅追加 `Db(#[from] rusqlite::Error)` 变体（open_central_db 需将 rusqlite 错误转 CoreError），dev-dependencies rusqlite 供集成测试命名 Connection 类型路径；均仅追加不改旧。rusqlite 已在 workspace.dependencies 基线声明（tech-design 选型 1），无新增未声明依赖。已在任务 Log 显式记录。

Build Verification：
- `cargo check -p mojian-core` → Finished, 0 error
- `cargo test -p mojian-core db` → 3 passed, 0 failed（12 表建库断言 / schema_version==1 断言 / 二次打开幂等断言含 schema_meta 单行 / foreign_keys=1）
- `cargo build --workspace` → exit 0，mojian 二进制生成

Builder Exit Criteria：5/5 通过

已知风险：无。v1 一次性建全表，后续列增改走同一 schema_meta 有序步骤迁移器（已预留 MIGRATIONS 步骤形状与版本戳 UPDATE 分支）。

## TASK-004 — 2026-07-07

变更文件：
- crates/mojian-core/src/project/manifest.rs（新增，ProjectManifest serde 模型 + write_manifest/read_manifest 读写 mojian.toml）
- crates/mojian-core/src/project/mod.rs（新增，register_project / load_project_state / update_project_spec + now_rfc3339/to_absolute 助手）
- crates/mojian-core/src/lib.rs（修改，仅追加 `pub mod project;` 与 project 侧 re-export）
- crates/mojian-core/Cargo.toml（修改，仅追加 uuid / time / serde / toml 依赖行，均 workspace = true）
- crates/mojian-core/tests/project.rs（新增，register→load + manifest 往返 + 错误路径集成测试）

实现摘要：在 mojian-core 落地项目登记与身份标记读写。manifest.rs 定义 `ProjectManifest { project_id, spec_version }`（`#[derive(Serialize, Deserialize)]`），write_manifest 用 toml 序列化写 `<dir>/mojian.toml`，read_manifest 读回，缺文件（io NotFound）或内容非法均映射为 CoreError::Io。mod.rs 的 register_project 生成 UUID v4 project_id，在单事务内插入 project 行（name / 绝对 path / created_at=updated_at RFC3339）与 project_state 行（sop_phase = SopPhase::StyleSampling.as_db_str() 即 "style_sampling"），返回 project_id；load_project_state 按 project_id 读 project_state.sop_phase 经 SopPhase::try_from 还原，无此项目由 query_row 的 QueryReturnedNoRows → CoreError::Db；update_project_spec 回填 project 行的 spec_version / spec_hash / updated_at 供后续 CLI（REQ-014）收口。

范围说明：error.rs 不在 Allowed Files，故 toml 序列化/解析失败复用既有 CoreError::Io 变体（以 io::ErrorKind::InvalidData 包裹并携带 mojian.toml 路径上下文），未新增错误变体、未改 error.rs。uuid/time/serde/toml 均已在 workspace.dependencies 基线声明（tech-design 选型 4/6），无新增未声明依赖。

Build Verification：
- `cargo check -p mojian-core` → Finished, 0 error
- `cargo test -p mojian-core project` → 4 passed, 0 failed（register→load 得 SopPhase::StyleSampling + path 绝对断言 / mojian.toml 往返 project_id+spec_version 一致 / 无此项目返回 CoreError::Db / 缺 manifest 返回 CoreError::Io）
- `cargo build --workspace` → exit 0

Builder Exit Criteria：6/6 通过

已知风险：无。update_project_spec 由本任务提供库函数，其在 mojian new 流程中的调用与 spec_version/spec_hash 实值回填由 TASK-005（SPEC 部署）/ TASK-006（CLI）收口。

## TASK-005 — 2026-07-07

变更文件：
- crates/mojian-cli/assets/spec/spec.toml（新增，version = "0.0.1-skeleton"，meta 不部署）
- crates/mojian-cli/assets/spec/CLAUDE.md（新增，占位）
- crates/mojian-cli/assets/spec/.claude/agents/.gitkeep（新增，占位）
- crates/mojian-cli/assets/spec/.claude/skills/.gitkeep（新增，占位）
- crates/mojian-cli/assets/spec/prompts/sop-1-style/README.md、sop-2-bible/README.md、sop-3-writing/README.md（新增，占位提示词包）
- crates/mojian-core/src/spec/mod.rs（新增，模块登记 + re-export）
- crates/mojian-core/src/spec/hash.rs（新增，blake3 tree hash）
- crates/mojian-core/src/spec/master.rs（新增，include_dir 嵌入 + 主副本 bootstrap + 权威 version/hash）
- crates/mojian-core/src/spec/deploy.rs（新增，部署 + hash 漂移覆盖）
- crates/mojian-core/src/lib.rs（修改，仅追加 `pub mod spec;` 与 spec 侧 re-export）
- crates/mojian-core/Cargo.toml（修改，仅追加 blake3 / include_dir 依赖行，均 workspace = true）
- crates/mojian-core/tests/spec.rs（新增，bootstrap/deploy/tree_hash/drift 集成测试）

实现摘要：在 mojian-core 落地 SPEC 主副本管理与部署机制（占位骨架）。hash.rs 的 tree_hash 对目录内每个文件按「相对路径（`/` 分隔）升序」拼接「相对路径 + 该文件内容 blake3 hex」再整体 blake3，得确定性 hex（顺序无关、内容敏感）；内部 hash_files 支持排除相对路径（供排除 spec.toml），deployed 与 authoritative 两侧对齐同一相对路径根，保证同构可比。master.rs 用 `include_dir!("$CARGO_MANIFEST_DIR/../mojian-cli/assets/spec")` 把占位载荷编译为 `EMBEDDED_SPEC`；ensure_master 在 `<data_dir>/spec/` 缺失时递归写出嵌入树形成权威主副本；authoritative_version 读主副本 spec.toml 的 version；authoritative_hash = 主副本除 spec.toml 外整棵树的 tree_hash。deploy.rs 的 deploy_spec 先删项目内 4 类部署目标（`.claude/agents`/`.claude/skills`/`CLAUDE.md`/`prompts`）再 1:1 递归拷载荷（跳过 spec.toml），返回 (version, hash)；sync_if_drifted 只对项目内部署目标条目实时重算 tree hash 与权威比对，不一致则覆盖重部署返回 (true, new_hash)，一致则不写返回 (false, hash)。

范围说明：include_dir 嵌入由 mojian-core 直接引用 `../mojian-cli/assets/spec`（tech-design 将嵌入落点委派本任务定义，assets 布局固定在 mojian-cli/assets/spec）；bootstrap/部署函数仍以 `&include_dir::Dir` 参数化，供 TASK-006 的 CLI 传入。error.rs 不在 Allowed Files，spec.toml 解析失败复用既有 CoreError::Io（InvalidData 包裹），未新增错误变体。blake3/include_dir 均已在 workspace.dependencies 基线声明（tech-design 选型 5/6），无新增未声明依赖。

Build Verification：
- `cargo check -p mojian-core` → Finished, 0 error, 0 warning
- `cargo build --workspace` → exit 0（触发依赖变更打包档）
- `cargo test -p mojian-core --test spec` → 6 passed, 0 failed
- `cargo test -p mojian-core` → 全绿（9+3+1+4+6 = 23 passed, 0 failed）

Builder Exit Criteria：6/6 通过

已知风险：无。CLI 侧（mojian new/status）对 ensure_master/deploy_spec/sync_if_drifted 的编排、DB spec_hash 回填由 TASK-006 收口。

## TASK-006 — 2026-07-07

变更文件：
- crates/mojian-cli/src/main.rs（重写，clap4 derive Cli/Command + 分发，顶层据 anyhow::Result 决定 ExitCode）
- crates/mojian-cli/src/spec_assets.rs（新增，`include_dir!("$CARGO_MANIFEST_DIR/assets/spec")` 嵌入占位主副本）
- crates/mojian-cli/src/commands/mod.rs（新增，子命令模块登记）
- crates/mojian-cli/src/commands/new.rs（新增，`mojian new <dir>` 有序 6 步）
- crates/mojian-cli/src/commands/status.rs（新增，`mojian status [--path]` 有序 3 步）
- crates/mojian-cli/src/commands/run.rs（新增，桩）
- crates/mojian-cli/src/commands/decide.rs（新增，桩，trailing_var_arg 忽略尾随参数）
- crates/mojian-cli/Cargo.toml（修改，仅追加 [dependencies]：mojian-core(path) / clap / anyhow / include_dir）
- crates/mojian-cli/tests/cli.rs（新增，驱动真实二进制的 5 项端到端集成测试）

实现摘要：把 TASK-001~005 的 core 能力编排成端到端命令面。main.rs 用 clap v4 derive 定义 `new/status/run/decide` 四子命令并分发，子命令统一返回 `anyhow::Result<()>`，`main` 返回 `ExitCode`（Ok→0、Err→打印 `错误：{err:#}` 到 stderr 并退 1）。new.rs：先以 `<dir>/mojian.toml` 是否存在判定并拒绝重复初始化（exit≠0），创建目录并取绝对路径（自研 absolutize，只在相对时 join cwd、不 canonicalize，避免 macOS `/var`→`/private/var` 漂移使 stdout 与 `$PROJ` 不符）；再 ensure 数据目录 + `ensure_master(&SPEC_ASSETS, master)` bootstrap + `open_central_db` 建库；`register_project`（事务，初始 style_sampling，name=目录 basename）→ `deploy_spec(master, project)` 得 (version, hash) → `update_project_spec` 回填 project 行（REQ-014）→ `write_manifest`；stdout 打印 project_id/绝对 path/style_sampling。status.rs：缺 mojian.toml → bail「非 mojian 项目」（exit≠0）；read_manifest 取 project_id；ensure_master + open_central_db；`sync_if_drifted(project, authoritative_hash, master)` 打开时 hash 覆盖，漂移则以 authoritative_version + new_hash 回填 DB；`load_project_state` 读 phase，打印 project(目录 basename)+phase。run/decide 打印「stub，将在 ITER-002 实现」并 exit 0，decide 以 `trailing_var_arg` 接受并忽略尾随参数。

范围说明：spec_assets.rs 独立在 CLI 侧用 include_dir 嵌入 `assets/spec`，经 core 参数化的 `ensure_master(&Dir, ...)` 注入 bootstrap（对齐 TASK-005 落位接口）。status 的项目名取自目标目录 basename（等价于 new 时存入 DB 的 name），以此避免在 CLI 直接依赖 rusqlite——Cargo.toml 仅追加任务 Allowed Files 枚举的 mojian-core/clap/anyhow/include_dir 四项，无越界新依赖。未触碰 crates/mojian-core/src/** 与 assets/**。

Build Verification：
- `cargo check --workspace` → Finished, 0 error
- `cargo build --workspace` → exit 0（触发依赖变更打包档）
- `cargo test -p mojian-cli` → tests/cli.rs 5 passed, 0 failed
- `cargo test --workspace` → 全绿（cli 5 + core lib 9 + db 3 + paths 1 + project 4 + spec 6 = 28 passed, 0 failed）
- 真实二进制端到端（MOJIAN_HOME=mktemp 隔离）：new→13 表/schema_version=1/manifest 含 project_id+spec_version/部署目标就位/spec.toml 不部署/project 行 spec 两列非空；status→style_sampling；篡改 CLAUDE.md 后 status 还原（tampered count=0）；run/decide 桩 exit 0；空目录 status exit 1「非 mojian 项目」；重复 new exit 1

Builder Exit Criteria：7/7 通过

已知风险：无。QA Verification（含 sqlite3 断言）待 qa-agent 接手真跑。
