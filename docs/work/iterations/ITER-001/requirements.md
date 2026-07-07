# Requirements — ITER-001

date: 2026-07-07
revision: 1
status: confirmed

## 目标

搭起 mojian 执行器的最小运行地基：建立 Cargo workspace、落地领域状态机类型与客户端中央 SQLite schema，让作者能用 `mojian new` 建出一个「运行环境就位」的项目（登记入中央 DB、部署好 SPEC、写出 `mojian.toml`），并用 `mojian status` 读回该项目当前的 SOP 阶段。达成「运行环境就位」里程碑，不触及创作生成循环。

## 正式需求

### 来源：issue#9（IMPL-1 执行器骨架）

- REQ-001：提供 Cargo workspace，含两个 crate —— `mojian-core`（库）与 `mojian-cli`（产出二进制 `mojian`）。
- REQ-002：`cargo check` 与 `cargo build --workspace` 均须通过。
- REQ-003：`mojian-core` 提供领域状态机类型 `SopPhase`，覆盖 SOP①②③ 的两级阶段（粗粒度 SOP 阶段 + 各 SOP 内部细粒度状态），语义对齐 storage.md「二、领域模型」。
- REQ-004：`mojian-core` 提供 `ChapterState`，含七个正常态 `Planned / SkeletonDrafting / SkeletonReview / ProseDrafting / ProseReview / Approved` 与作废态 `Void`。
- REQ-005：所有领域枚举变体命名与 `docs/naming.md` **逐字一致**（代码与文档不得出现两套叫法）。
- REQ-006：客户端中央 DB 为 SQLite（`central.db`），须按 storage.md「五、DB 表设计」建立全部表：`project`、`project_state`、`reference_book`、`volume`、`batch`、`chapter`、`artifact_ref`、`bible_version`、`void_record`、`stat`、`config`、`schema_meta`。
- REQ-007：DB 具备基于 `schema_meta.schema_version` 的迁移能力（首版初始化即写入 schema 版本）。
- REQ-008：`mojian new <dir>` 须：建出项目目录、在中央 DB 的 `project` 表登记该项目（含 `project_id`）、并在项目目录写出 `mojian.toml`（至少含 `project_id`）。
- REQ-009：`mojian status` 须读回中央 DB，输出该项目当前的 SOP phase。
- REQ-010：`mojian run` 与 `mojian decide` 本迭代仅留桩（stub）——命令存在、可被调用，但不执行实际业务逻辑；被调用时打印「stub，将在 ITER-002 实现」提示并以成功状态（exit 0）正常退出。

### 来源：issue#10（IMPL-2 SPEC 部署 + 运行环境）

- REQ-011：客户端侧提供 SPEC 主副本管理：`spec/` 目录（sop-1 / sop-2 / sop-3）并带版本与内容 hash。本迭代 `spec/` 内放**占位骨架**（仅验证「部署 + hash 覆盖」通路），不放真实提示词内容——真实提示词待 SOP #5/#6/#7 定稿后由后续迭代填充。
- REQ-012：`mojian new` 在建项目时，把客户端 SPEC 主副本部署进项目目录（`.claude/agents`、`.claude/skills`、`CLAUDE.md`、`prompts`）。
- REQ-013：打开项目时比对「项目 SPEC 缓存 hash」与「客户端权威 hash」：不一致则直接覆盖重部署；一致则不写（选项 A：项目内 SPEC 为纯可弃缓存）。
- REQ-014：`project` 表的 `spec_version` / `spec_hash` 须与项目实际部署的 SPEC 保持一致。

## 约束

- 命名遵循 `docs/naming.md`：crate 目录 `crates/mojian-core`、`crates/mojian-cli`（kebab-case，`mojian-` 前缀）；枚举/类型 `PascalCase`；状态机 phase 名与三个 SOP 设计逐字对齐。
- 机器状态一律存客户端中央 DB（按 `project_id` 分区），项目目录内**不得**存放机器状态；项目与状态靠 `mojian.toml` 的 `project_id` 关联。
- 项目内 SPEC 为纯可弃缓存，权威永远以客户端为准（选项 A）。
- 本迭代不产生任何 token 花费类行为（不调 SDK、不做切片、不做 LLM 检查）。
- **客户端数据目录位置**：`central.db` / `spec/` 主副本 / 日志目录须落在真实默认路径。默认 `~/.mojian/`；具体路径策略（Linux 遵循 XDG、macOS 用 Application Support 等平台标准目录）由 design-agent 在技术设计阶段确定。本约束只固定「必须有一个真实默认位置、默认 `~/.mojian/`」，不锁定平台细节。
- **新建项目初始 phase**：`mojian new` 后 `mojian status` 首次输出的初始 SOP phase 为 `style_sampling`（SOP① 首阶段）。

## 不在范围内

- SDK 调用、切片装配器、人机决定循环实际逻辑、客观检查器、VOID / 过期检测、日志文件（generation/decision/check）写入、题材 SPEC 变体管理、stat 表数据产出。
- `docs/product.md` 的内容补全（归档阶段处理）。

## 待确认项

- [x] 需求边界是否清晰？（本迭代 = 「运行环境就位」，创作循环全部排除）—— 已确认，边界清晰
- [x] 有无遗漏的关键场景？—— 已确认无遗漏
- [x] 约束是否合理？—— 已确认
- [x] **客户端数据目录位置** —— 已决定：默认 `~/.mojian/`，平台标准目录细节交 design-agent（见约束节）
- [x] **SPEC 主副本初始内容** —— 已决定：本迭代放占位骨架，仅验证部署 + hash 覆盖通路（见 REQ-011）
- [x] **新建项目初始 phase** —— 已决定：`style_sampling`（见约束节）
- [x] **桩命令行为** —— 已决定：打印「stub，将在 ITER-002 实现」并正常退出（见 REQ-010）
