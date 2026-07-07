# Tech Design · Storage

> 存储的落地（设计草案，planned）。先立概念模型（客户端 / 项目），再讲每层物理落盘与 DB 表设计。作用域总览见 `overview.md`；运行时读写见 `engine.md`。

## 一、概念模型：程序是状态源，项目是运行环境

两个作用域，各自装什么：

| | **客户端（程序，全局）** | **项目（运行环境，一个目录）** |
|---|---|---|
| SPEC | **权威副本**（版本化，带 hash） | 部署缓存（可弃，启动 hash 覆盖） |
| SSOT | — | **创作内容**（过程产物 + 结果产物） |
| 机器状态/日志/统计 | **中央 DB**（按 project_id 分区） | 无 |
| 配置 | 全局默认 + 各项目覆盖项 | 无 |

三对关系（storage 之前缺的骨架）：

- **SPEC ↔ 项目**：SPEC 是「怎么做」，不属于任何单一创作内容。权威只在客户端；项目里那份是纯可弃缓存，启动时对不上 hash 就覆盖。题材风格覆盖由客户端**按项目**管理，不在项目里改（选项 A）。
- **项目 ↔ SSOT**：SSOT 是**项目的本体**——项目（运行环境）里人直读直改的就是这棵创作内容树。SSOT 是人最终要的东西。
- **项目 ↔ 状态/配置**：状态 = 「这个项目走到 SOP 哪一步」，逻辑上跟着项目、物理上存客户端中央 DB（按 `project_id`）。配置默认随程序、项目只存覆盖项。

## 二、客户端（中央）

```
mojian 客户端数据目录/
  spec/                     # ★ SPEC 权威副本（版本化，folder 形态，git 友好）
    sop-1-style/ sop-2-bible/ sop-3-writing/
  central.db                # ★ 中央 DB：项目登记 + 各项目状态/日志/统计/配置
  defaults.toml             # 全局默认配置（红线阈值等）
```

- SPEC 权威用**文件夹**（文本，版本化/覆盖/贡献都比塞 DB 自然）；`central.db` 里只记 SPEC 版本 + hash，不存正文。
- 题材 SPEC 变体也在客户端管理（哪个项目派生/选用哪个 SPEC 包）。

## 三、项目（运行环境）

```
{项目}/
  # —— SPEC 部署缓存（可弃，claude 原生读）——
  .claude/agents/*.md   .claude/skills/   CLAUDE.md   prompts/

  # —— SSOT（创作内容，人直读直改）——
  references/book/*.txt                        输入：参考小说原文（人放）
  materials/{book}/skeleton.md                 过程产物：抽取骨架
  creative/creative-brief.md                   过程产物：借鉴定位
  creative/creative-vision.md                  过程产物：创意愿景
  bible/*.md                                   ❓过程/结果：圣经九件套（开放问题①）
  outline/*.md                                 结果产物：大纲
  volumes/{arc}/plan.md                        过程产物：章节计划（内容）
  volumes/{arc}/chapters/{ch}-skeleton.md      ❓过程/结果：骨架（开放问题①）
  volumes/{arc}/chapters/{ch}.md               结果产物：正文
  mojian.toml                                  身份标记：project_id（+ 已部署 SPEC 版本）；非机器状态
```

项目里**没有 DB**——机器状态全在客户端 `central.db`，靠 `mojian.toml` 的 `project_id` 关联。

### SSOT 文件格式（内容契约，由 SPEC 定义）

| 文件 | 关键结构 |
|---|---|
| 抽取骨架 `materials/{book}/skeleton.md` | 分块（~5万字/块）→ 情节节拍 · 爽点与钩子标注 · 人物出场 · 章末悬念类型（节奏统计入 DB，不塞此文件） |
| `creative-brief.md` | 值得借鉴的爽点系统/升级结构/节奏模型，逐项注明出处与理由（客观） |
| `creative-vision.md` | 题材定位 · 主角设定 · 金手指 · 预期规模（主观决策，不重复 brief） |
| 圣经九件套 `bible/*.md` | 世界观规则 · 爽点系统 · 金手指 · 主角弧度 · 人物档案 · 时间线 · 文风(style) · 禁忌 · 伏笔账本（schema 见开放问题③） |
| 大纲 `outline/*.md` | 全书大纲 + 各卷大纲，只展开不创新 |
| 章节计划 `plan.md`（逐章条目） | `story_scope` · `protagonist_goal` · `obstacle` · `chapter_turn` · `reader_payoff` · `key_characters_state`（纯内容，**无状态字段**——状态在客户端 DB） |
| 骨架 `{ch}-skeleton.md` | ≤1000 字：场景序列 · 场景时序与因果 · 伏笔处理 · 结尾钩子 |
| 正文 `{ch}.md` | 完整章节，严格依从骨架 |

### 切片约束（与 `engine.md` 装配器耦合）

段级切片（如「圣经 style.md 里骨架相关的那一段」）要求 SSOT 文件有**机器可寻址的稳定结构**（命名小标题锚点），否则只能整文件塞入。原则：只在「高频被读 + 体量大」的产物上加结构——圣经用命名段落、大纲按卷/章加锚点；正文除「前一章整篇」外不作输入切入。见开放问题②。

## 四、DB 表设计（客户端 `central.db`）

> 列名 `snake_case`，schema 定稿后确定。除 `project` 外，各表均带 `project_id` 外键（按项目分区）。

| 表 | 存什么 | 作用 |
|---|---|---|
| `project` | project_id、路径、名称、创建时间、部署的 SPEC 版本+hash | 项目登记（中央唯一） |
| `project_state` | project_id、phase、当前卷、各游标、时间戳 | 状态机的输入 |
| `chapter` | project_id、id、arc、status、verify_flag、deviation、骨架/正文文件 hash | 章节流水线 |
| `batch` | project_id、id、arc、章节集、status | 调度单位 |
| `artifact_ref` | project_id、path、kind(spec/过程/结果)、content_hash、version | **DB↔SSOT 的桥**（靠 hash 认变更） |
| `generation_log` | project_id、step、spec 路径+hash、本次读的输入切片及其 hash、token 进/出、成本 | 可复现 + AP-002 token 对账 |
| `bible_version` | project_id、版本、原因、触发源(void/人工)、时间 | 圣经版本化 |
| `void_record` | project_id、章节、原因、影响范围、时间 | VOID 记录 |
| `decision_log` | project_id、关卡、判定(CONFIRMED/REVISE/VOID)、人写的评论/补充信息、时间 | 人的决定入库 |
| `check_result` | project_id、章节/批、检查项、指标值、通过与否 | 客观红线结果 |
| `stat` | project_id、对话占比、章字数、钩子密度等 | 参考书风格画像 + 写作对账 |
| `config` | project_id、键、覆盖值 | 项目配置**覆盖项**（默认在 `defaults.toml`） |

### DB↔SSOT 的桥：hash 游标

`artifact_ref.content_hash` + `generation_log` 里记录的输入切片 hash，共同支撑「过期检测」（见 `domain-model.md` 的 VOID 机制）。product.md 里已有的 `product_md_hash` 游标就是这个思路的先例。

### schema 版本与升级

`central.db` 带 `schema_version`；程序升级时**只在客户端跑一次迁移**，不用逐项目处理（集中的红利，见 `overview.md` 升级模型）。

## 开放问题

1. **过程/结果产物的判据**：判据是「读者/作者最终要的东西」(→ 结果) vs「为产出它而搭的脚手架」(→ 过程)，还是别的？据此**圣经**、**骨架**各归哪一类？
2. **能否接受圣经/大纲遵守一套轻量小标题锚点约定**（切片可行性的前提）。
3. 圣经九件套的标准 schema：哪些字段结构化供程序校验、哪些自由文本供模型阅读？（#6 开放问题①）

> 原「节奏统计入 DB」开放问题已随「机器状态全集中客户端」定案：入 `central.db` 的 `stat` 表。
