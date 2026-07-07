# Technical Design — ITER-001

date: 2026-07-07
revision: 1
status: confirmed

## Overview

本迭代把项目级技术设计基线（overview / storage / engine）的「运行环境就位」部分落成可编译代码：一次性建立 Cargo workspace（`mojian-core` 库 + `mojian-cli` 二进制），在 `mojian-core` 落地两级领域状态机类型（`SopPhase` / `ChapterState`），把 storage.md「五」的 12 表 DDL 做成基于 `schema_meta` 的建库/迁移实现，并实现 `mojian new`（建项目 + 中央 DB 登记 + SPEC 部署 + 写 `mojian.toml`）与 `mojian status`（读中央 DB 输出 SOP phase），`run`/`decide` 留桩。本设计只做实现级选型、模块布局、数据模型落地、CLI 契约与 SPEC 部署机制，不重新发明架构，不触及 token 花费面（SDK / 切片 / 检查器）。

## 选型对比

以下每项均为「实现级」选型；架构分界（客户端=状态源 / 项目=运行环境 / 选项 A）已由基线锁定，不在此重议。

### 1. SQLite 驱动 — rusqlite（推荐） vs sqlx

- **选项 A — `rusqlite`（带 `bundled` feature）（推荐）**
  - 实现路径：`rusqlite = { version = "0.31", features = ["bundled"] }`；同步 API，单 `Connection` 打开客户端 `central.db`。
  - 优点：同步、零运行时依赖，契合「本地单用户 CLI、单库、无并发服务」；`bundled` 编译内置 SQLite，跨平台无需系统 libsqlite；`execute_batch` 直接跑 DDL 脚本，与 storage.md 的裸 SQL DDL 一一对应，改动成本最低。
  - 缺点：无编译期 SQL 校验；`bundled` 需要 C 编译器参与构建（见 DevOps 影响）。
- **选项 B — `sqlx`**
  - 实现路径：async + `sqlx::SqlitePool`，需引入 tokio 运行时。
  - 优点：编译期查询校验、内建迁移器、连接池。
  - 缺点：为一个本地单连接 CLI 引入 async 运行时与连接池属过度设计；其迁移器与 `schema_meta` 显式表模型不天然契合（见第 3 项）。
- 与基线约束契合度：基线要求「机器状态存客户端中央 SQLite，按 project_id 分区」，无并发/网络需求 → 同步 rusqlite 完全够用且最轻。

### 2. CLI 框架 — clap v4 derive（推荐）

- 实现路径：`clap = { version = "4", features = ["derive"] }`，`#[derive(Parser)]` + `#[derive(Subcommand)]` 定义 `new/status/run/decide`。
- 优点：Rust 生态事实标准，derive 声明式子命令、自动 `--help`/版本、错误信息友好；后续 ITER-002 扩 `run/decide` 参数面平滑。
- 备选（不采用）：`argh`（更轻但功能弱、社区小）、`structopt`（已并入 clap，弃用）。

### 3. 数据目录定位 — `directories` crate + 分层解析（推荐）

- 实现路径：`directories = "5"`；解析顺序 **`MOJIAN_HOME` 环境变量 → 平台标准目录（`ProjectDirs::from("", "", "mojian")` → Linux `$XDG_DATA_HOME/mojian`（默认 `~/.local/share/mojian`）、macOS `~/Library/Application Support/mojian`） → 兜底 `~/.mojian/`**。
- 优点：满足需求约束「用平台标准目录（Linux XDG / mac Application Support）」；`MOJIAN_HOME` 覆盖使集成测试可指向隔离临时目录（本迭代健壮测试的关键，避免污染真实 `~`）；`~/.mojian/` 作为 home 不可解析时的最终兜底，对齐约束「默认 `~/.mojian/`」的语义。
- 说明：需求约束原文「默认 `~/.mojian/`」在本方案中被实现为**兜底位**而非各平台字面路径——把字面 `~/.mojian/` 提升为跨平台首选会违背同一句约束里「用平台标准目录」的要求。二者取「平台标准目录优先、`~/.mojian/` 兜底」以同时满足。（Round 3 人工已确认此解析顺序。）
- 备选（不采用）：所有平台字面 `~/.mojian/`（无视平台规范，与约束「平台标准目录」冲突）。

### 4. `mojian.toml` 序列化 — serde + toml（推荐）

- 实现路径：`serde = { version = "1", features = ["derive"] }` + `toml = "0.8"`；`#[derive(Serialize, Deserialize)] struct ProjectManifest { project_id: String, spec_version: String }`。
- 优点：生态标准；`mojian.toml` 人可读、字段最小（约束仅要求至少含 `project_id`，本设计附带 `spec_version` 便于人工核对）。

### 5. 内容 hash — blake3（推荐） vs sha2

- **选项 A — `blake3`（推荐）**：`blake3 = "1"`；`blake3::hash(bytes)` 单调用，最快，hex 输出，API 极简。
- **选项 B — `sha2`（SHA-256）**：更「标准」、更眼熟，但更慢、API 略繁。
- 选择理由：spec_hash 仅用于**内部**部署缓存一致性比对，不是需与外部系统互操作的标准化校验和；无 SHA 标准化诉求 → 取更快更简的 blake3。
- 目录树 hash 策略：`spec_hash` = 对**部署载荷树**按「相对路径升序」拼接「相对路径 + 该文件内容 blake3」再整体 blake3，得到确定性 tree hash（顺序无关、内容敏感）。

### 6. 其余基础 crate

| 用途 | crate | 理由 |
|------|-------|------|
| `project_id` 生成 | `uuid = { version = "1", features = ["v4"] }` | storage.md 规定 `project_id` 为 UUID |
| 时间戳（`created_at`/`updated_at` TEXT） | `time = { version = "0.3", features = ["formatting"] }` | 生成 RFC3339 字符串写入 TEXT 列，轻量、无历史包袱 |
| 应用层错误（CLI） | `anyhow = "1"` | 顶层错误聚合与友好报错 |
| 领域/库错误（core） | `thiserror = "1"` | `mojian-core` 暴露有类型的 `CoreError` |
| 嵌入 SPEC 占位骨架 | `include_dir = "0.7"` | 把占位 SPEC 主副本编译进二进制，首次运行时落地到 `<data_dir>/spec/`，保证「真实默认位置有真实（占位）内容」 |

## 采用方案

选项组合：**rusqlite(bundled) + clap4(derive) + directories(分层解析) + serde/toml + blake3 + uuid/time/anyhow/thiserror/include_dir**。

核心理由：本迭代是「结构 + 建项目 + 读状态 + SPEC 部署」的工程奠基，无并发、无网络、无 token 面，选型以「同步、最轻、跨平台无系统依赖、与 storage.md 裸 DDL 直接对应」为准绳。全部选型将写入 workspace 根 `Cargo.toml` 的 `[workspace.dependencies]` 统一版本管理。

## 涉及模块

Workspace 布局：

```
Cargo.toml                      # [workspace] + [workspace.dependencies]
crates/
  mojian-core/                  # 库
    Cargo.toml
    src/
      lib.rs                    # 门面 re-export
      error.rs                  # CoreError (thiserror)
      paths.rs                  # 客户端数据目录解析 + 路径助手
      domain/
        mod.rs
        sop_phase.rs            # SopPhase（Level 1 粗粒度 + 内含细粒度语义）+ DB 文本映射
        chapter_state.rs        # ChapterState（七态 + Void）+ DB 文本映射
        extract_status.rs       # ExtractStatus（pending/extracting/extracted，SOP① 抽取游标；辅助类型）
      db/
        mod.rs                  # open_central_db(path) -> Connection（打开即跑迁移）
        schema.rs               # SCHEMA_VERSION 常量 + v1 建表 SQL（12 表）
        migrate.rs              # 基于 schema_meta.schema_version 的迁移运行器
      project/
        mod.rs                  # register_project / load_project_state（DB 侧）
        manifest.rs             # mojian.toml 读写（ProjectManifest）
      spec/
        mod.rs
        master.rs              # 主副本定位 + 首次落地嵌入骨架 + 权威 version/hash
        deploy.rs              # 部署（copy 主副本→项目）+ hash 比对覆盖
        hash.rs                # 部署载荷树 tree hash（blake3）
  mojian-cli/                   # 二进制 mojian
    Cargo.toml
    assets/spec/                # include_dir 嵌入的占位 SPEC 主副本（部署载荷树）
      spec.toml                #   version = "0.0.1-skeleton"（meta，不部署）
      CLAUDE.md                #   占位
      .claude/agents/.gitkeep  #   占位
      .claude/skills/.gitkeep  #   占位
      prompts/sop-1-style/README.md
      prompts/sop-2-bible/README.md
      prompts/sop-3-writing/README.md
    src/
      main.rs                  # clap 解析 + 分发
      commands/
        mod.rs
        new.rs                 # mojian new <dir>
        status.rs              # mojian status
        run.rs                 # 桩
        decide.rs              # 桩
```

| 模块路径 | 变更类型 | 说明 |
|---------|---------|------|
| `Cargo.toml`（workspace 根） | 新增 | workspace 成员 + 统一依赖版本 |
| `crates/mojian-core/src/domain/*` | 新增 | `SopPhase` / `ChapterState` / `ExtractStatus` + DB 文本映射（REQ-003/004/005） |
| `crates/mojian-core/src/db/*` | 新增 | 12 表建库 + `schema_meta` 迁移（REQ-006/007） |
| `crates/mojian-core/src/project/*` | 新增 | project/project_state 登记 + `mojian.toml` 读写（REQ-008） |
| `crates/mojian-core/src/spec/*` | 新增 | SPEC 主副本 / 部署 / hash 覆盖（REQ-011/012/013/014） |
| `crates/mojian-core/src/paths.rs` | 新增 | 客户端数据目录分层解析（约束：默认 `~/.mojian/`） |
| `crates/mojian-cli/src/*` | 新增 | `new`/`status` 实现 + `run`/`decide` 桩（REQ-008/009/010） |
| `crates/mojian-cli/assets/spec/*` | 新增 | 占位 SPEC 主副本（REQ-011） |

## API 变更

本迭代无网络 API；「API 面」= CLI 命令契约。二进制名 `mojian`（`mojian-cli` 产出）。

### `mojian new <dir>`
- 入参：`<dir>`（必填，项目目录路径，相对/绝对均可，内部转绝对路径）。
- 前置：确保客户端数据目录存在；若 `central.db` 不存在则建库并跑迁移；若 `<data_dir>/spec/` 不存在则从嵌入骨架落地主副本。
- 行为（有序）：
  1. 校验 `<dir>`：不存在则创建；若已存在且含 `mojian.toml` → 报错退出（非 0），拒绝重复初始化。
  2. 生成 `project_id`（UUID v4），`name` = 目录 basename，`path` = 绝对路径。
  3. 在一个 DB 事务内插入 `project` 行与 `project_state` 行（`sop_phase = "style_sampling"`，即 SOP① 首阶段，对齐约束）。
  4. 部署 SPEC 主副本 → 项目目录（`deploy_spec`），得部署 `spec_version` / `spec_hash`。
  5. 用 `spec_version` / `spec_hash` 更新 `project` 行（满足 REQ-014）。
  6. 写 `mojian.toml`（`{ project_id, spec_version }`）。
- 输出（stdout）：`project_id`、绝对 `path`、初始 phase `style_sampling`。exit 0。
- 认证：无（本地 CLI，无鉴权面；对齐基线——鉴权非本工具关注点）。

### `mojian status [--path <dir>]`
- 入参：`--path <dir>`（可选，默认当前工作目录）。
- 行为：
  1. 在目标目录读 `mojian.toml` 取 `project_id`；无 `mojian.toml` → 报错「非 mojian 项目」（非 0 退出）。
  2. 执行「打开项目」SPEC 同步（见「SPEC 部署 + hash 覆盖机制」）：比对权威 hash 与项目实际部署树 hash，不一致则覆盖重部署并更新 DB `spec_hash`；一致则不写。
  3. 从 `central.db` 按 `project_id` 读 `project_state.sop_phase`，输出该项目名 + 当前 SOP phase。exit 0。
- 认证：无。

### `mojian run` / `mojian decide [args...]`（桩）
- 行为：打印 `stub，将在 ITER-002 实现`，以 exit 0 正常退出（REQ-010）。`decide` 接受但忽略尾随参数，保证命令面存在、可被调用。

## 数据模型变更

### 建库与迁移（REQ-006/007）
- `db/schema.rs` 持有 storage.md「五」的**逐字 12 表 DDL**（`project`、`project_state`、`reference_book`、`volume`、`batch`、`chapter`、`artifact_ref`、`bible_version`、`void_record`、`stat`、`config`、`schema_meta`），以 `execute_batch` 在一个事务内建全部表。`SCHEMA_VERSION: i64 = 1`。
- `db/migrate.rs` 迁移运行器（**自研，键于 `schema_meta` 表**，不用 PRAGMA `user_version`——storage.md 明确要求以 `schema_meta.schema_version` 驱动，`rusqlite_migration`/`refinery` 均用 `user_version`，故不采用）：
  1. 打开连接，`PRAGMA foreign_keys = ON`。
  2. 查 `schema_meta` 是否存在且有行：
     - 无（全新库）→ 跑 v1（建 12 表）+ `INSERT INTO schema_meta(schema_version) VALUES (1)`。
     - 有 → 读 `schema_version`，从该版本按有序编号 migration 步骤逐步升级（本迭代仅 v1，无后续步骤）。
  3. 迁移整体在事务内，失败回滚。
- `open_central_db(path)` = 打开连接 → 跑迁移 → 返回 `Connection`，是所有 DB 访问的唯一入口。

### 枚举 ↔ DB 文本值映射（REQ-005 逐字一致）
- 变体命名 `PascalCase`（naming.md）；DB / TOML 存 `snake_case` 文本；映射集中在各枚举的 `as_db_str()` + `TryFrom<&str>`，代码与文档单一事实源（避免两套叫法）。

| 枚举 | 变体（PascalCase，逐字对齐 naming.md / storage.md） | DB 文本值（snake_case） |
|------|---------------------------------------------------|-------------------------|
| `SopPhase`（Level 1 粗粒度，覆盖 SOP①②③） | `StyleSampling` / `StyleExtracting` / `BriefDrafting` / `VisionDrafting`（SOP①）；`BibleBuilding` / `BibleCheck` / `BibleVerify` / `OutlineExpand` / `OutlineVerify`（SOP②）；`Writing`（SOP③，卷内循环见 Level 2） | `style_sampling` / `style_extracting` / `brief_drafting` / `vision_drafting` / `bible_building` / `bible_check` / `bible_verify` / `outline_expand` / `outline_verify` / `writing` |
| `ChapterState`（SOP③ 章节，Level 2 最细） | `Planned` / `SkeletonDrafting` / `SkeletonReview` / `ProseDrafting` / `ProseReview` / `Approved` / `Void` | `planned` / `skeleton_drafting` / `skeleton_review` / `prose_drafting` / `prose_review` / `approved` / `void` |
| `ExtractStatus`（SOP① 抽取游标，辅助） | `Pending` / `Extracting` / `Extracted` | `pending` / `extracting` / `extracted` |

- REQ-003「两级阶段」落地：`SopPhase` 承载 Level 1 全部粗粒度阶段（含 SOP① 抽取的 `StyleExtracting` 粗阶段）；SOP① 抽取的块级细粒度游标由 `ExtractStatus` + `reference_book.block_cursor` 承载；SOP③ 章节细粒度由 `ChapterState` + `chapter.status` 承载。三者共同覆盖 storage.md「二」的两级模型。本迭代仅要求类型存在且能与 DB 文本互转，不实现状态转移函数（转移逻辑属 ITER-002+）。
- migration 影响：本迭代为 schema v1 全新建库，无既有数据迁移；后续列增改由 v2+ 有序步骤处理（见风险）。

## 前端变更

无（纯 CLI 工具，无前端）。

## SPEC 部署 + hash 覆盖机制（占位骨架）

- **主副本落地（bootstrap）**：`assets/spec/` 经 `include_dir` 嵌入二进制；首次运行发现 `<data_dir>/spec/` 缺失时，把嵌入树写出到 `<data_dir>/spec/`，形成客户端权威主副本（REQ-011，占位骨架，仅验证通路）。
- **部署载荷**：主副本 `<data_dir>/spec/` 下除 `spec.toml`（版本 meta，不部署）外的整棵树即「部署载荷」，顶层条目对齐 engine.md 部署目标：`.claude/agents/`、`.claude/skills/`、`CLAUDE.md`、`prompts/`（SOP 包 `sop-1-style` / `sop-2-bible` / `sop-3-writing` 置于 `prompts/` 下）。**主副本布局即部署布局，1:1 拷贝**，使 hash 比对与覆盖语义最简。
- **权威版本/hash**：`spec_version` 读自主副本 `spec.toml` 的 `version`；权威 `spec_hash` = 对部署载荷树算 blake3 tree hash（见选型 5）。
- **部署（`mojian new`，REQ-012）**：把载荷树递归拷入项目根，写 `project.spec_version` / `project.spec_hash`（REQ-014）。
- **打开时 hash 覆盖（选项 A，REQ-013）**：打开项目（本迭代由 `mojian status` 触发，未来由 `run`/open 触发）时，重算**项目实际部署树**的 blake3 tree hash 作为「项目 SPEC 缓存 hash」，与客户端权威 hash 比对：
  - 不一致 → 直接覆盖重部署，并更新 `project.spec_hash` / `spec_version`；
  - 一致 → 不写（项目内 SPEC 为纯可弃缓存）。
  - 「项目缓存 hash」采用**实时重算部署树**而非在项目内存 hash 标记文件——对齐约束「项目目录内不得存放机器状态」，且能同时检测「客户端主副本升级」与「项目内被手改」两种漂移。
- 覆盖策略：先删除项目内旧部署目标路径再写入（保证删除类变更也被覆盖），仅限部署目标条目，不动 SSOT 目录。

## 依赖与风险

- **技术依赖（新增 crate）**：`rusqlite(bundled)` / `clap` / `directories` / `serde` / `toml` / `blake3` / `uuid` / `time` / `anyhow` / `thiserror` / `include_dir`。均为 Rust 生态成熟库，MIT/Apache 许可。
- **已知风险**：
  1. **一次性 12 表 schema 的迁移演进**：v1 一把梭建全表，后续任一列增改都要走自研 `schema_meta` 迁移器。缓解：迁移器从设计起即按「有序编号步骤 + 事务 + 失败回滚」实现，v1 只是首个步骤，为 v2+ 预留形状；`open_central_db` 为唯一入口，升级只此一处。
  2. **跨作用域一致性**：机器状态在客户端 DB、SPEC 部署在项目文件系统，二者可能不一致（如 DB 已登记但部署失败）。缓解：`mojian new` 中 DB 登记走事务；部署与 `mojian.toml` 写在登记之后，任一步失败即报错并暴露已登记的 `project_id` 供人工清理；打开时的 hash 覆盖是自愈通路（部署漂移会被下次 open 纠正）。本迭代不做跨作用域分布式事务（过度设计），以「幂等重部署 + 明确报错」兜底。
  3. **首个 workspace 奠基选型锁定**：本迭代确立的驱动/框架/目录策略被后续所有迭代继承，改选型成本高。缓解：选型以「同步最轻 + 生态标准 + 无系统依赖」为准，均为长期主流；依赖版本集中在 `[workspace.dependencies]` 便于统一升级。
  4. **委派落点为 design-agent 的判断**（数据目录解析顺序、SPEC 主副本=部署布局的 1:1 映射、项目缓存 hash 采用实时重算）——Round 3 人工已逐项确认。
- **测试健壮性**：`MOJIAN_HOME` 环境覆盖使 `cargo test` 指向隔离临时目录，`new`/`status`/部署/hash 覆盖可端到端集成测试而不触真实 `~`。
- **不引入的新依赖（对齐基线约束）**：不引入 async 运行时（tokio）、不引入 ORM、不引入 HTTP/网络栈、不调用任何 LLM SDK、不做切片/检查器——本迭代零 token 花费面。

## PRD 影响

`docs/product.md` 目前为空，本设计不改产品边界。相对项目级技术设计基线（overview/storage/engine），本迭代新增以下**实现级细化**，建议由 archivist-agent 在迭代关闭时归档进 tech-design：

1. 客户端数据目录解析策略：`MOJIAN_HOME` → 平台标准目录（XDG / Application Support）→ `~/.mojian/` 兜底（基线仅约定「默认 `~/.mojian/`」，此为落地细化）。
2. SPEC 主副本布局 = 部署布局（1:1 拷贝，SOP 包置于 `prompts/` 下）+ 主副本 `spec.toml` 承载 `version`（基线只描述 `spec/sop-N` 与部署目标，此为二者的具体映射）。
3. 「项目 SPEC 缓存 hash」采用实时重算部署树（而非项目内 hash 标记），blake3 tree hash 定义（相对路径升序 + 逐文件内容 hash）。
4. 选型基线：rusqlite(bundled) / clap4 / directories / serde+toml / blake3 —— 建议记入 tech-design 的技术栈约束。

（以上均为对基线的兼容性细化，无冲突项。Round 3 人工已认可，PRD 影响留待归档处理。）

## DevOps 影响

`docs/devops.md` 当前仅有 `vcs_platform` / `review_policy`，无启动/端口/账号声明。本设计不新增端口、账号、健康检查端点或常驻服务。但引入以下**构建期要求**，建议归档时记入 devops：

- 构建工具链：需 `cargo`（Rust stable）；`rusqlite` 的 `bundled` feature 会编译内置 SQLite，需系统具备 C 编译器（macOS: Xcode CLT；Linux: cc/clang）。
- 构建/验证命令：`cargo check` 与 `cargo build --workspace`（REQ-002 验收口）；测试 `cargo test`（可配合 `MOJIAN_HOME` 隔离）。
- 运行产物：单二进制 `mojian`；运行期写 `<data_dir>/`（默认按平台标准目录，`MOJIAN_HOME` 可覆盖），无外部服务依赖。

（Round 3 人工已认可，DevOps 影响留待归档处理。）

## 待确认项

- [x] 选型决策是否正确？（rusqlite(bundled) / clap4 / directories / serde+toml / blake3 / include_dir 等）—— Round 3 确认，无异议
- [x] **数据目录解析顺序** `MOJIAN_HOME → 平台标准目录 → ~/.mojian/ 兜底` 是否可接受？—— Round 3 确认（MOJIAN_HOME 便于隔离测试）
- [x] **SPEC 主副本布局 = 部署布局（1:1 拷贝，SOP 包置于 `prompts/` 下）** 是否可接受？—— Round 3 确认
- [x] **「项目 SPEC 缓存 hash」采用实时重算部署树**（不在项目内存 hash 标记，以守约束「项目内不存机器状态」）是否可接受？—— Round 3 确认
- [x] API（CLI 命令契约）是否覆盖所有 REQ？（REQ-001~014 ↔ 模块/命令映射见「涉及模块」「API 变更」）—— 终审核对通过
- [x] 风险评估是否充分？（12 表迁移演进 / 跨作用域一致性 / workspace 奠基）—— 终审核对通过
- [x] PRD / DevOps 影响是否需要在本迭代内同步处理，还是留待归档？—— Round 3 决定：留待归档
