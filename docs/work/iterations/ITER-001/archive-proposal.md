# Archive Proposal — ITER-001

date: 2026-07-07
revision: 1
status: confirmed

## 迭代成果概览

完成任务：6 个（全部 done）
取消任务：0 个
关键产出：
- TASK-001：Cargo workspace（`mojian-core` 库 + `mojian-cli` 二进制 `mojian`）+ 统一依赖基线 + `paths.rs` 数据目录分层解析 + `CoreError`。
- TASK-002：领域三枚举 `SopPhase` / `ChapterState` / `ExtractStatus` + DB 文本互转（逐字对齐 naming.md / storage.md）。
- TASK-003：客户端中央 SQLite `central.db` 建库（storage.md「五」12 表）+ `schema_meta` 迁移器 + `open_central_db` 唯一入口。
- TASK-004：project 登记（`project` / `project_state`）+ `mojian.toml`（`project_id` / `spec_version`）读写。
- TASK-005：SPEC 主副本（占位骨架，`include_dir` 嵌入）/ 1:1 部署 / blake3 tree hash + 打开时 hash 漂移覆盖。
- TASK-006：`mojian-cli` 命令面——`new` / `status` 实现，`run` / `decide` 桩；workspace 28 测试通过、真实二进制端到端 QA 通过。

里程碑「运行环境就位」达成（捆绑 issue#9 + #10）。

## 对 docs/product.md 的修改建议

`docs/product.md` 当前为空（仅 `# Product`）。本迭代首次为其补入「已落地功能」的 What 层描述。

### 章节：全文（product.md 首次成文）

**原文：**

```
# Product
```

**新文：**

```
# Product

> mojian = SPEC（提示词资产）+ 确定性执行器（Rust）。理念与架构见 `docs/tech-design/`。本文记录**已落地**的用户可见能力（What）。

## 已落地功能

### 运行环境就位（ITER-001，里程碑）

mojian 执行器骨架落地，作者可以建出并检视一个「运行环境就位」的小说项目：

- **`mojian new <dir>`**：建项目目录、在客户端中央 DB 登记项目（分配 `project_id`）、把 SPEC 主副本部署进项目、写出 `mojian.toml`；初始 SOP 阶段为 `style_sampling`。
- **`mojian status [--path <dir>]`**：读回客户端中央 DB，输出该项目当前的 SOP 阶段；打开时按 hash 比对，SPEC 部署漂移会被自动覆盖还原。
- **`mojian run` / `mojian decide`**：本阶段为桩（打印「stub，将在 ITER-002 实现」并以 exit 0 正常退出），创作生成循环尚未接入。

客户端中央 SQLite（`central.db`）承载全部机器状态（12 表 + `schema_meta` 迁移）；项目目录内只有 SPEC 部署缓存与 `mojian.toml`，不含机器状态。SPEC 本迭代为**占位骨架**，仅验证「部署 + hash 覆盖」通路，真实提示词待 SOP #5/#6/#7 定稿后填充。

## 尚未落地

创作生成循环（SDK 调用、切片装配、人机决定循环、客观检查器、VOID / 过期检测、日志文件写入、题材 SPEC 变体、统计产出）——见 `docs/tech-design/engine.md`，规划自 ITER-002 起。
```

**原因：** product.md 空白，缺少「当前产品能做什么」的 What 层事实；本迭代交付了首个可用命令面，需沉淀为产品现状。
**对应任务：** TASK-001~006（整体里程碑）

## 对项目级技术设计基线的修改建议

基线为 `docs/tech-design/`（overview / storage / engine），此前全篇标注「设计草案 planned / 未由代码证明」。本迭代把「运行环境就位」部分落成可编译并端到端验证的代码，产生 4 项实现级细化，建议归档如下（均为对基线的兼容性细化，无冲突项；Round 3 人工已认可）。

### 建议 1 — storage.md「三、客户端」：SPEC 主副本布局 = 部署布局（1:1）+ spec.toml

**原文：**

```
  spec/                     # ★ SPEC 权威副本（版本化，folder 形态，git 友好）
    sop-1-style/ sop-2-bible/ sop-3-writing/
```

**新文：**

```
  spec/                     # ★ SPEC 权威副本（= 部署布局，1:1；folder 形态，git 友好）
    spec.toml               #   版本 meta：version（不部署）
    CLAUDE.md  .claude/     #   部署载荷：顶层条目对齐 engine.md 部署目标
    prompts/sop-1-style/ prompts/sop-2-bible/ prompts/sop-3-writing/
```

**原因：** 实现确定「主副本布局 = 部署布局，1:1 拷贝」，SOP 包置于 `prompts/` 下（而非 `spec/` 顶层），并由 `spec.toml` 的 `version` 承载权威版本。原图把 SOP 包画在 `spec/` 顶层，与实际部署载荷树不一致，需校正为实况。
**对应任务：** TASK-005

### 建议 2 — storage.md「三、客户端」：数据目录位置解析策略

**原文：**（在该节末两条要点之后追加）

```
- SPEC 权威用**文件夹**（文本，版本化/覆盖/贡献都比塞 DB 自然）；`central.db` 里只记 SPEC 版本 + hash，不存正文。
- 题材 SPEC 变体也在客户端管理（哪个项目派生/选用哪个 SPEC 包）。
```

**新文：**

```
- SPEC 权威用**文件夹**（文本，版本化/覆盖/贡献都比塞 DB 自然）；`central.db` 里只记 SPEC 版本 + hash，不存正文。
- 题材 SPEC 变体也在客户端管理（哪个项目派生/选用哪个 SPEC 包）。
- **数据目录位置解析（ITER-001 落地）**：按 `MOJIAN_HOME` 环境变量 → 平台标准目录（Linux `$XDG_DATA_HOME/mojian`，默认 `~/.local/share/mojian`；macOS `~/Library/Application Support/mojian`）→ 兜底 `~/.mojian/` 的顺序定位。`MOJIAN_HOME` 使集成测试可指向隔离临时目录，不污染真实 `~`。
```

**原因：** 基线只约定「默认 `~/.mojian/`」，未明确跨平台解析顺序；实现落地为「平台标准目录优先、`~/.mojian/` 兜底、`MOJIAN_HOME` 可覆盖」，需记入基线。
**对应任务：** TASK-001

### 建议 3 — engine.md「启动执行流」：项目缓存 hash 实时重算 + blake3 tree hash 定义

**原文：**

```
2. 比对项目 SPEC 缓存 hash vs 客户端权威 → 不一致直接覆盖重部署（选项 A：项目内 SPEC 纯可弃）
```

**新文：**

```
2. 比对项目 SPEC 缓存 hash vs 客户端权威 → 不一致直接覆盖重部署（选项 A：项目内 SPEC 纯可弃）
   - **项目缓存 hash 实时重算（ITER-001 落地）**：不在项目内存 hash 标记文件（守「项目内不存机器状态」约束），每次打开时实时重算项目实际部署树的 blake3 **tree hash**——按「相对路径升序」拼接「相对路径 + 该文件内容 blake3」再整体 blake3（顺序无关、内容敏感）。既能检测客户端主副本升级，也能检测项目内被手改两种漂移。
```

**原因：** 基线只说「比对 hash」，未定义 hash 算法与「不落地标记文件、实时重算」的策略；实现明确采用 blake3 tree hash + 实时重算，需记入基线。
**对应任务：** TASK-005

### 建议 4 — overview.md：技术栈基线（新增小节）

**原文：**（在「## 升级模型」小节末条要点后追加新小节）

```
- **默认配置升级**：默认值随程序；项目只存覆盖项，没覆盖的自动吃新默认。
```

**新文：**

```
- **默认配置升级**：默认值随程序；项目只存覆盖项，没覆盖的自动吃新默认。

## 技术栈基线（ITER-001 落地）

执行器为 Rust Cargo workspace：`crates/mojian-core`（库）+ `crates/mojian-cli`（产出二进制 `mojian`）。奠基选型（后续迭代继承，锁定成本高，均取生态主流）：

| 关注点 | 选型 | 理由 |
|--------|------|------|
| SQLite 驱动 | `rusqlite`（`bundled`） | 同步、零运行时依赖、编译内置 SQLite 跨平台无系统依赖；`execute_batch` 与裸 DDL 直接对应 |
| DB 迁移 | 自研 `schema_meta` 迁移器 | storage.md 要求以 `schema_meta.schema_version` 驱动（非 PRAGMA `user_version`）；有序编号步骤 + 事务 + 失败回滚 |
| CLI 框架 | `clap` v4（derive） | Rust 生态标准，声明式子命令 |
| 数据目录定位 | `directories` + 分层解析 | 平台标准目录 + `MOJIAN_HOME` 覆盖（见 storage.md「三」） |
| 序列化 | `serde` + `toml` | `mojian.toml` 人可读 |
| 内容 hash | `blake3`（tree hash） | 仅内部部署缓存比对，取更快更简，无 SHA 标准化诉求 |
| 嵌入 SPEC 骨架 | `include_dir` | 占位主副本编译进二进制，首次运行落地到 `<data_dir>/spec/` |
| 其余 | `uuid` / `time` / `anyhow` / `thiserror` | project_id / 时间戳 / 应用层与库错误 |

全部版本集中在 workspace 根 `[workspace.dependencies]` 统一管理。本迭代零 token 花费面：不引入 async 运行时（tokio）/ ORM / HTTP 网络栈 / LLM SDK。
```

**原因：** 基线三篇均无「技术栈」记录；本迭代确立的选型被后续所有迭代继承，属需固化的架构级约束。
**对应任务：** TASK-001~006（选型贯穿全迭代）

## 对 docs/devops.md 的修改建议（需人工确认后应用）

> 说明：`docs/devops.md` 属人工维护的规范文件，archivist-agent 合同禁止自行改写。以下仅作**建议**列出，消除 building 阶段暴露的「无构建/验证声明」gap；是否应用、由谁应用，请人工在 CONFIRMED 时明确授权。

### 建议 5 — devops.md：新增「## Build Verification」节

**原文：**

```
# DevOps

## Config

vcs_platform: gh
review_policy: human_required
```

**新文：**

```
# DevOps

## Config

vcs_platform: gh
review_policy: human_required

## Build Verification

Rust workspace（`cargo` stable）。`rusqlite` 的 `bundled` feature 会编译内置 SQLite，构建期需系统具备 C 编译器（macOS: Xcode CLT；Linux: cc/clang）。

- 检查：`cargo check`
- 构建：`cargo build --workspace`（REQ-002 验收口）
- 测试：`cargo test`（可配 `MOJIAN_HOME=<临时目录>` 指向隔离数据目录，避免污染真实 `~`）

运行产物：单二进制 `mojian`；运行期写 `<data_dir>/`（默认平台标准目录，`MOJIAN_HOME` 可覆盖），无外部服务依赖。
```

**原因：** design 阶段「DevOps 影响」记录了构建期要求（cargo + C 编译器、验证命令、单二进制运行），但 devops.md 尚无对应声明；building 阶段各 agent 反复手记 `cargo build --workspace` 验证命令，缺统一出处。
**对应任务：** TASK-001（工具链）/ 全迭代（验证命令）

## 不归档的 done 任务

无任务需整体排除——6 个任务的产出均已在 What 层（product.md）或实现级细化（tech-design / devops）中归档。以下**实现细节**有意不上升到 PRD/基线（属内部实现，非产品或架构约束）：

| 内容 | 原因 |
|------|------|
| `CoreError` 变体划分、`thiserror`/`anyhow` 分层 | 内部错误实现细节，非产品/架构约束 |
| 模块目录切分（`domain/` `db/` `project/` `spec/`）与函数签名 | 内部实现布局，随实现演进，不进 PRD |
| 各枚举 `as_db_str()` / `TryFrom<&str>` 映射代码形态 | 映射的事实（枚举↔DB 文本）已由 storage.md 表承载；代码形态属实现 |
| 测试结构（`tests/*.rs`、`MOJIAN_HOME` 隔离手法） | 验证手段，`MOJIAN_HOME` 覆盖已在 devops 建议中记录 |

## 协议复核建议（本轮变更是否导致框架文档需要精简）

`protocol/builder_driver.md` 与各 agent 合同位于 `.esr_harnass/`（安装产物禁区）与 `.claude/agents/`（安装产物），本项目不得改写。本轮为产品项目（mojian）的常规归档，未触及框架协议。
- 无需精简（N/A：框架文档由框架版本管理，不在本仓库维护范围）。

## 待确认项

- [ ] product.md 修改建议是否同意？（首次成文：已落地功能 + 尚未落地）
- [ ] tech-design 修改建议 1~4 是否同意？（storage 主副本布局校正 + 数据目录解析 / engine hash 定义 / overview 技术栈基线）
- [ ] devops.md 建议 5 是否同意、并授权由谁应用？（规范文件，需人工明确授权）
- [ ] 不归档清单是否合理？
- [ ] 协议复核结论（无需精简）是否认可？
