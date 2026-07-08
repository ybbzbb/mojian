# Technical Design — ITER-002

date: 2026-07-07
revision: 1
status: confirmed
final-check: Round 1 CONFIRMED — passed 2026-07-07

> 本迭代把 ITER-001 留下的 `run` / `decide` 桩换成真逻辑，打通「生成闭环」的**机制通路**：切片装配 + 输入契约 manifest（IMPL-3）、无头 `claude` 子进程 SDK 调用、三条 JSONL 中的 generation / decision 两条、`run → decide → run` 状态机推进（IMPL-4）。占位 SPEC 只验机制，不产真实创作物；SDK 命令可注入 mock（硬约束）。设计不重新发明架构，只在 `engine.md` / `storage.md` 既定基线上做实现级选型。

## Overview

在 `mojian-core` 新增四个模块——`sdk`（生成命令抽象 + 无头 `claude` 子进程 + JSON 解析）、`context`（manifest 解析 + 符号引用解析 + 切片 + bundle 组装 + write_scope）、`log`（generation/decision JSONL 追加写）、`engine`（`next_action` 纯函数状态机 + `apply_generation` / `apply_decision` 推进 + `Verdict` 判定），并新增 `state` 模块承载运行时 DB 行读写（project_state 推进 / 关卡标记 / chapter 状态 / void_record / artifact_ref）。CLI `run.rs` / `decide.rs` 从桩转实现，`status.rs` 扩展显示卡点。生成命令默认 `claude`，经 `MOJIAN_CLAUDE_CMD` 环境变量或 `GenerationRunner` trait 注入替换为测试假命令。**无 DB schema 迁移**（现有 12 表 + `project_state.cursors` JSON 足够）；新增唯一依赖 `serde_json`。

占位 SPEC 需带一个最小但真实、可被解析的输入契约 manifest + 一个可跑通的「步」：本迭代把 SOP① 头部 `brief_drafting` 步接成完整的 Generate 步（装配 → 调 mock SDK → 写 generation.jsonl → 停在 `brief` 关卡），`decide brief CONFIRMED/REVISE` 推进 / 回喂，端到端满足裁决② 的「一次 `run → decide → run`」验收深度。

## 选型对比

### 选型 1 — SDK 调用抽象与 mock 注入（硬约束）

**选项 A — `GenerationRunner` trait + `MOJIAN_CLAUDE_CMD` 双层注入（推荐）**

- 实现路径：`sdk` 模块定义 `trait GenerationRunner { fn run(&self, bundle: &Bundle) -> Result<SdkResponse, CoreError>; }`；默认实现 `ClaudeCliRunner` 用 `std::process::Command` 拼 `claude -p <prompt> --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope...>`，其**基础命令名**读自 `MOJIAN_CLAUDE_CMD`（缺省 `claude`）。`engine` 核心逻辑签名接 `&dyn GenerationRunner`。
- 优点：两个测试层都被覆盖——① CLI 端到端测（在 `mojian-cli/tests` 内驱动**真实二进制**，无法把 Rust trait 注进被 spawn 的子进程）靠 `MOJIAN_CLAUDE_CMD` 指向一个假命令脚本满足裁决② 的「注入 mock SDK 端到端测」；② `mojian-core` 单元测靠 `FakeRunner` 实现，不 spawn 任何进程（快、确定、无脚本/shebang 平台差异）。契合约束「真实 `claude` 为默认、外部命令可替换」。
- 缺点：比纯 env 方案多一个 trait 与一次分发；两条注入路径需各自留测试。
- 与基线契合度：对齐 `engine.md`「Rust → 无头 `claude` 子进程，引用已部署 agent + 传参数」；trait 让「状态机是代码」（AP-001）与生成 IO 解耦。

**选项 B — 仅 `MOJIAN_CLAUDE_CMD` 环境变量替换（备选）**

- 实现路径：不设 trait，`sdk::run_generation` 直接 spawn `Command`，所有测试（含 core 单元测）都指向假命令脚本。
- 优点：类型更少、只一条注入路径。
- 缺点：core 单元测被迫 spawn 进程 + 依赖磁盘上的可执行脚本（shebang / Windows 平台敏感、慢、易脆）；生成逻辑与进程 IO 无法解耦，装配/日志的纯逻辑测试也得起子进程。

**采用：选项 A。** requirements 约束「外部命令必须可替换、QA 须能在不触达真实 claude 前提下验证全链路」在 A 下最稳：E2E 走 env 假命令、core 走 trait 假实现，两层都不花 token。

### 选型 2 — 子进程执行库

**选项 A — `std::process::Command`（推荐）**：标准库、零新依赖、同步（对齐 overview.md「不引入 async 运行时」）、`output()` 一次抓 stdout/stderr/exit code 足够。
**选项 B — 第三方（`duct` / `subprocess`）**：管道/组合更顺手，但本场景只需「跑一次、抓 stdout JSON」，引入依赖不划算。
**采用：选项 A。**

### 选型 3 — 输入契约 manifest 的承载形态

**选项 A — 部署 SPEC 内的 TOML sidecar manifest（推荐）**

- 实现路径：每个可跑步骤在部署 SPEC 里带一份 sidecar，如 `.claude/agents/<agent>.manifest.toml`，声明 `agent` / `inputs`（符号引用数组）/ `write`（白名单）/ `output_contract`。Rust `context` 模块读它做装配；`claude` 仍原生读同目录的 `<agent>.md`。占位 SPEC 新增一个最小 `brief-agent.md` + `brief-agent.manifest.toml`。
- 优点：**零新解析依赖**——`toml` + `serde` 已是基线（ITER-001 用于 `mojian.toml` / `spec.toml`）；sidecar 与 agent 正文物理分离，Rust 读契约、claude 读提示词，各取所需；manifest 随 SPEC 一起部署 + 走既有 hash 覆盖，天然版本化。
- 缺点：与 `engine.md` 里 YAML frontmatter 的**示意**写法不同（见「PRD 影响」）。
- 与基线契合度：`engine.md`「manifest 用符号引用、`write:` 喂 `write_scope`」的**语义**完全保留；只是格式取 TOML sidecar。

**选项 B — agent `.md` 的 YAML frontmatter（备选）**

- 优点：与 `engine.md` 示例逐字一致、单文件。
- 缺点：需引入 `serde_yaml`（额外依赖 + 另一套解析）；且要把 frontmatter 从 claude 会连带读到的 agent 正文里切出来解析，边界更糙。

**采用：选项 A。** `engine.md` 的 YAML 是**示意**（「写在 agent frontmatter 或 sidecar」原文即给了 sidecar 选项），非格式承诺；取 TOML sidecar 守住 overview.md「本迭代不新增无谓依赖」的克制。

### 选型 4 — JSON 解析 / JSONL 写入依赖

**选项 A — `serde_json`（推荐 / 唯一可行）**：解析 `claude --output-format json` 的 stdout，且 generation/decision 事件用 `serde::Serialize` 结构体 + `serde_json::to_string` 逐行写 JSONL。属 serde 生态标准件，与既有 `serde` 直接协作。
**选项 B — 手写 JSON**：解析 claude 输出不现实、易错。
**采用：选项 A。** 这是本迭代**唯一新增依赖**，加入 workspace `[workspace.dependencies]`。不引入 `serde_yaml` / async / HTTP / LLM SDK。

### 选型 5 — 符号引用解析器

**选项 A — 手写小解析器（推荐）**：符号文法 `<source>.<selector>[:<params>][#anchor]`（如 `bible.style#skeleton`、`plan.chapters:{batch}`、`prev_skeleton:{ch-1}`），按 `.` / `:` / `#` 切分，确定性、零依赖。
**选项 B — 正则 / 解析框架**：过重，文法简单不值当。
**采用：选项 A。**

### 选型 6 — run/decide 逻辑归属

**选项 A — 落 `mojian-core::engine`（推荐）**：`next_action` 为纯函数、`apply_*` 收敛 DB 写，core 内可完整单元测；CLI 层薄（解析参数 → 调 core → 打印）。守 AP-001「状态机是代码不是散文」。
**选项 B — 逻辑写在 CLI**：状态机散进命令处理函数，难测、与 `status` 复用差。
**采用：选项 A。**

## 采用方案

选型 1-A / 2-A / 3-A / 4-A / 5-A / 6-A。要点：

- `sdk`：`GenerationRunner` trait + `ClaudeCliRunner`（`std::process::Command`，基础命令读 `MOJIAN_CLAUDE_CMD`）+ `serde_json` 解析 `SdkResponse`。
- `context`：TOML sidecar manifest → 手写符号解析 → 段级（`#anchor`）/ 整文件切片 → 组装五字段 `Bundle`（含 `inputs` 内联 + `decision.jsonl` 评论回喂）。
- `log`：`serde_json` 逐行追加写 `generation.jsonl` / `decision.jsonl` 到 `<data_dir>/logs/{project_id}/`。
- `engine` + `state`：纯函数 `next_action` + `apply_generation` / `apply_decision`，落 `project_state.sop_phase` / `cursors`（关卡标记）/ `chapter.status` / `void_record` / `artifact_ref`。
- 占位 SPEC 加一个可跑的 `brief_drafting` Generate 步 + `brief` 人工关卡，端到端跑通一次 `run → decide → run`。

## 涉及模块

| 模块路径 | 变更类型 | 说明 |
|---------|---------|------|
| `crates/mojian-core/src/sdk/mod.rs` | 新增 | `GenerationRunner` trait、`Bundle`（五字段）、`SdkResponse`（result/cost/usage） |
| `crates/mojian-core/src/sdk/claude_cli.rs` | 新增 | `ClaudeCliRunner`：`std::process::Command` 拼参 + `MOJIAN_CLAUDE_CMD` + JSON 解析 + 错误处理 |
| `crates/mojian-core/src/context/manifest.rs` | 新增 | TOML sidecar manifest 模型 + 读取（`inputs` / `write` / `output_contract`） |
| `crates/mojian-core/src/context/symbol.rs` | 新增 | 符号 `<source>.<selector>[:{params}][#anchor]` 解析 + 按状态代入 `{arc_id}`/`{batch}`/`{ch-1}` |
| `crates/mojian-core/src/context/slice.rs` | 新增 | 段级（`#anchor` 稳定小标题抽取）/ 整文件切片 + 内容 blake3 hash |
| `crates/mojian-core/src/context/assemble.rs` | 新增 | `assemble_bundle`：解析 manifest → 解析符号 → 切片 → 回喂 decision 评论 → 组五字段 |
| `crates/mojian-core/src/log/mod.rs` | 新增 | `append_generation` / `append_decision` + 事件结构体；`read_decision_comments`（REQ-011 回喂） |
| `crates/mojian-core/src/engine/mod.rs` | 新增 | `Action` / `Verdict` 枚举、`next_action`（纯函数）、`apply_generation` / `apply_decision`、phase→action 映射表 |
| `crates/mojian-core/src/state/mod.rs` | 新增 | 运行时 DB 行读写：`advance_sop_phase` / `set_gate` / `clear_gate` / `load_chapter` / `update_chapter_status` / `insert_void_record` / `upsert_artifact_ref` |
| `crates/mojian-core/src/lib.rs` | 修改 | 导出上述新模块公共项 |
| `crates/mojian-core/src/error.rs` | 修改 | 新增错误变体（子进程失败 / JSON 解析失败 / manifest 非法 / 符号无法解析 / 关卡状态不匹配） |
| `crates/mojian-cli/src/commands/run.rs` | 修改 | 桩 → 真逻辑：定位项目 → 循环 `next_action` → Generate 执行 → 撞关卡停 |
| `crates/mojian-cli/src/commands/decide.rs` | 修改 | 桩 → 真逻辑：解析 `<关卡> <判定> [目标] [--comment\|--file]` → 写 decision.jsonl → `apply_decision` |
| `crates/mojian-cli/src/commands/status.rs` | 修改 | 扩展：卡在关卡时显示「关卡 + 等什么决定」 |
| `crates/mojian-cli/assets/spec/.claude/agents/brief-agent.md` | 新增 | 占位可跑步骤的 agent 提示词（占位） |
| `crates/mojian-cli/assets/spec/.claude/agents/brief-agent.manifest.toml` | 新增 | 占位步骤的最小真实输入契约（`inputs` 含整文件 + `#anchor` 段级各一，`write` 白名单） |
| `Cargo.toml`（workspace） | 修改 | `[workspace.dependencies]` 加 `serde_json`；`mojian-core` 依赖引入 |

> 部署目标含 `.claude/agents`（见 `deploy.rs` `DEPLOY_TARGETS`），新增 agent + manifest sidecar 落该目录即随现有部署 / hash 覆盖机制生效，无需改部署代码。

## API 变更

> mojian 是本地 CLI，无网络端点。以下为**命令面 API**（对齐 `engine.md`「人机决定接口」）与 **core 库 API**。

### CLI `mojian run`

- 形态：`mojian run [--path <dir>]`
- 行为：定位项目（读 `mojian.toml` 取 `project_id`）→ 打开时 hash 覆盖（复用 `sync_if_drifted`）→ 循环 `engine::next_action(state)`：
  - `Action::Advance` → 纯推进 `sop_phase`（占位：SOP① `style_sampling` / `style_extracting` 为 no-op 直进，见「依赖与风险 · 占位简化」）。
  - `Action::Generate { agent, manifest }` → `assemble_bundle` → `runner.run(bundle)` → `append_generation` → 置 `brief` 关卡标记 → 停。
  - `Action::HumanGate { gate }` → 停机，打印卡点。
  - `Action::Idle` → 无可跑动作，正常退出（占位深层 phase 未接线时的诚实出口）。
- 认证：无（本地命令）。

### CLI `mojian decide`

- 形态：`mojian decide <gate> <verdict> [target] [--comment "..." | --file <path>]`
  - `<verdict>` ∈ `CONFIRMED` | `REVISE` | `VOID`；`REVISE` / `VOID` 带 `[target]`（`CH-xxx` / 批 id）。
- 行为：校验当前确在 `<gate>`（否则非 0 退出）→ `append_decision`（关卡 / 判定 / 目标 / 评论或文件内容 / 时间戳）→ `engine::apply_decision`：
  - `CONFIRMED` → 清关卡 + 推进（`brief` → `vision_drafting`）。
  - `REVISE` → 清关卡 + 回退对应细粒度状态（`brief` 关卡回 `brief_drafting`；章节关卡回 `skeleton_drafting`）；评论留待下次装配回喂（REQ-011）。
  - `VOID <CH>` → 写 `void_record` + `chapter.status: void → planned`（裁决③ 最小语义，不级联 / 不过期检测）。
- 认证：无。

### CLI `mojian status`（扩展）

- 在 ITER-001「project / phase」输出基础上：若 `project_state.cursors` 含关卡标记或存在 `skeleton_review` 章节，追加打印「卡在 `<gate>` 关卡 / 等待判定：CONFIRMED\|REVISE\|VOID」。

### core 库 API（`mojian-core` 新增导出）

- `sdk::GenerationRunner`（trait）、`sdk::ClaudeCliRunner`、`sdk::Bundle`、`sdk::SdkResponse`
- `context::assemble_bundle(conn, project_id, project_dir, manifest_path) -> Result<Bundle, CoreError>`
- `log::append_generation(project_id, &GenerationEvent)` / `log::append_decision(project_id, &DecisionEvent)` / `log::read_decision_comments(project_id, gate, target) -> Vec<String>`
- `engine::next_action(&RunState) -> Action`、`engine::apply_generation(...)`、`engine::apply_decision(...)`、`engine::Verdict`
- `state::{advance_sop_phase, set_gate, clear_gate, load_chapter, update_chapter_status, insert_void_record, upsert_artifact_ref}`

**`Bundle` 五字段（对齐 `engine.md`）：** `agent`（部署 agent 相对路径）、`spec_slice`（本步 SPEC 切片）、`inputs`（切片后 SSOT 结构化参数 + 回喂的人类评论）、`write_scope`（`Vec<PathBuf>` 白名单，由 manifest `write` 推导）、`output_contract`（期望产出 + done 信号形状）。

**`SdkResponse`（REQ-005）：** `result: String`、`cost: Option<f64>`（`total_cost_usd`）、`usage_in: Option<u64>` / `usage_out: Option<u64>`（`usage.input_tokens` / `usage.output_tokens`）。字段以 serde `rename` + `Option` 容错解析。

## 数据模型变更

**无 DB schema 迁移。** 现有 12 表（storage.md「五」）+ `project_state.cursors TEXT`（JSON）足以承载本迭代全部机器状态。`SCHEMA_VERSION` 保持 1，`MIGRATIONS` 表不追加步骤。

落库映射：

| 机器状态 | 落点 | 说明 |
|---------|------|------|
| Level-1 SOP 推进 | `project_state.sop_phase` | `advance_sop_phase` 更新 |
| 人工关卡标记（SOP① 顺序关卡） | `project_state.cursors` JSON 增 `pending_gate` 字段 | 用既有 TEXT 列，**免迁移**；`run` 见标记即停，`decide` 清除 |
| Level-2 章节推进 / 章节关卡 | `chapter.status`（`skeleton_review` 本身即关卡） | 章节关卡不需额外标记 |
| VOID 记录 | `void_record`（章节 / 原因 / 影响范围） | 裁决③：只记录 + `void → planned` |
| 输入切片 hash | `artifact_ref`（path / kind / content_hash） | 与 generation.jsonl 中的切片 hash 同源，供 ITER-003 过期检测（留位不留债） |

**JSONL 日志（文件，非 DB；storage.md「六」）：** 追加写 `<data_dir>/logs/{project_id}/`（`logs_dir()` 已就绪，本迭代加写入器）：

- `generation.jsonl`：`step` · `agent` · `spec_path` + `spec_hash` · `inputs`（切片列表及各自 `content_hash`）· `token_in` / `token_out` · `cost` · `ts`。
- `decision.jsonl`：`gate` · `verdict` · `target`（章 / 批）· `comment`（或 `--file` 内容）· `ts`。
- `check.jsonl`：**本迭代不写**（裁决①，归 ITER-003 / IMPL-5）。

## 前端变更

无（纯 CLI，无前端）。

## 依赖与风险

**技术依赖：**
- 新增 `serde_json`（workspace 依赖）——解析 claude 输出 JSON + 写 JSONL；serde 生态标准件，与既有 `serde` 协作。
- 运行时依赖 `claude` 可执行（默认命令）；测试用 `MOJIAN_CLAUDE_CMD` 指向假命令，不触达真实 claude、不花 token。

**已知风险：**
- **真实 `claude --output-format json` 的字段 schema 未在本迭代验证**：本迭代只跑 mock（裁决②），`SdkResponse` 解析按文档形态（`result` / `total_cost_usd` / `usage.input_tokens|output_tokens`）+ `Option` 容错。真实 claude 版本差异待接真实调用（ITER-003+）时以实测收口——影响面被 mock 隔离，属留位不留债。
- **占位简化 · SOP① 前置 phase**：`style_sampling` / `style_extracting` 本迭代作 no-op 直进占位（真实采样 / 分块抽取属 SOP① SPEC，出范围），仅 `brief_drafting` 为完整 Generate 步、`brief` 为完整关卡。诚实标注，不是隐藏债。
- **端到端验收深度**：CLI E2E 覆盖裁决② 的 `run → decide → run`（brief 通路，mock SDK，REVISE 带评论演示 REQ-011 回喂 / CONFIRMED 演示推进）；**章节级 VOID / REVISE**（目标 CH、`void → planned`、`void_record`）因需 volume/batch/chapter 行种子，由 `mojian-core` 单元 / 集成测在临时 DB 直接种子 chapter 行覆盖（`state` 模块提供 chapter 读写 helper），不强行塞进 CLI E2E。
- **关卡标记入 `cursors` JSON**：SOP① 顺序关卡的 pending 状态借 `project_state.cursors` JSON 承载以免本迭代加列；ITER-003 若关卡语义变复杂可提升为独立列（低风险、可演进）。

**接缝（留位不留债，与 ITER-003 对齐）：**
- `next_action` 在 Generate 与关卡之间**预留客观检查步位**（裁决①）：本迭代生成后直接置关卡 / 落库，检查步为空过；ITER-003 / IMPL-5 在此插入 Rust 检查器 + `check.jsonl`，不需改动 `next_action` 骨架。
- `generation.jsonl` 输入切片 hash + `artifact_ref.content_hash` 同源写入：本迭代只写不查；ITER-003 / IMPL-6 的圣经改动 → 输入 hash 过期检测直接消费这批数据，无需回填。

**不引入的新依赖（对齐 overview.md「零 token 花费面」）：** 不引入 async 运行时（tokio）/ ORM / HTTP 网络栈 / LLM SDK / `serde_yaml`。

## PRD 影响

以下由 archivist-agent 在迭代关闭时归档进项目级设计基线：

- **`engine.md` · manifest 格式**：`engine.md` 用 YAML frontmatter 示意输入契约；本迭代落地取**TOML sidecar**（`<agent>.manifest.toml`），语义（符号引用 / `write:` → `write_scope`）不变。建议归档时把 `engine.md` 的示意措辞对齐为「TOML sidecar（YAML 为等价示意）」。
- **`overview.md` · 技术栈基线**：新增 `serde_json`（JSON 解析 + JSONL 序列化）。建议在「技术栈基线」表追加一行。
- **`engine.md` / `storage.md` · 关卡持久化**：SOP① 顺序关卡的 pending 标记落 `project_state.cursors` JSON，未新增列。属实现细节，与基线不冲突，供归档时知会。

## DevOps 影响

无。生成命令为本地子进程（默认 `claude`，测试经 `MOJIAN_CLAUDE_CMD` 注入假命令），无新增端口 / 账号 / 服务 / 健康检查端点；不要求 `docs/devops.md` 变更。`infra.md` 为空、无静态环境约束冲突。（备注：真实生成运行需 `claude` 在 PATH，属运行时工具而非部署服务配置。）

## 待确认项

- [x] 选型决策是否正确？（尤其：SDK 双层注入 `GenerationRunner` trait + `MOJIAN_CLAUDE_CMD`；manifest 取 TOML sidecar 而非 YAML frontmatter；新增 `serde_json`）
- [x] API 设计是否覆盖所有 REQ？（REQ-001~012 → manifest 解析 / 双粒度切片 / 五字段 bundle / 子进程 + JSON / generation.jsonl / run 停关卡 / status 显卡点 / decide 三判定 + decision.jsonl / 评论回喂 / run→decide→run）
- [x] 风险评估是否充分？（真实 claude JSON schema 留待 ITER-003 收口；SOP① 前置 phase 占位直进；章节级 VOID/REVISE 走 core 测而非 CLI E2E）
- [x] PRD / DevOps 影响是否需要本迭代内同步处理？（建议：仅归档时同步 `engine.md` manifest 措辞 + `overview.md` 技术栈基线加 `serde_json`；DevOps 无影响）
