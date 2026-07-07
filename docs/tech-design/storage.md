# Tech Design · Storage

> 三层存储（SPEC / SSOT / DB）的落地（设计草案，planned）。三层定义见 `overview.md`；运行时如何部署与读取见 `engine.md`。

## SPEC —— 客户端主副本 → 项目部署副本

SPEC 不散在项目里由框架「安装」，而是由 **mojian 客户端集中持有主副本**，新建项目时**部署**一份进项目。

```
mojian 客户端数据目录/
  spec/                     # ★ SPEC 主副本（版本化，git 友好）
    sop-1-style/            #   风格抽取的各步骤 SPEC + 写作指南
    sop-2-bible/
    sop-3-writing/
  config.toml               # 全局默认配置（红线阈值等）
  registry.db               # 客户端级：SPEC 版本索引、项目登记

           │  mojian new <项目>  部署
           ▼
{项目}/
  .claude/agents/*.md       # 部署副本：SOP agent SPEC（claude 原生识别）
  .claude/skills/           # 部署副本：skills
  CLAUDE.md                 # 部署副本：项目级指令
  prompts/                  # 部署副本：写作指南等 agent 会 Read 的 SPEC 资产
```

- **主副本位置**：客户端**文件夹**（推荐——SPEC 是文本，folder 版本化/覆盖/贡献都比塞进 DB 自然）；`registry.db` 只**索引版本**，不存正文。
- **部署**：`mojian new` 把主副本拷进项目的 `.claude/agents` / `.claude/skills` / `CLAUDE.md` / `prompts`，使项目内 `claude` 能原生读到。
- **覆盖链**：项目本地部署副本可被作者直接改（题材风格包覆盖）；升级时保留本地覆盖。
- **版本记录**：DB `generation_log` 记下每次生成用的 SPEC 路径 + hash（可复现）。
- VOID 的语义写在对应 SOP 的 SPEC 里（程序只记录，见 `domain-model.md`）。

## SSOT —— 创作内容文件（人直读直改）

项目内目标布局（planned；具体目录名待首个实现迭代确认）：

```
references/book/*.txt                        输入：参考小说原文（人放）
materials/{book}/skeleton.md                 过程产物：抽取骨架
creative/creative-brief.md                   过程产物：借鉴定位
creative/creative-vision.md                  过程产物：创意愿景
bible/*.md                                   ❓过程/结果：圣经九件套（见开放问题①）
outline/*.md                                 结果产物：大纲
volumes/{arc}/plan.md                        过程产物：章节计划（内容）
volumes/{arc}/chapters/{ch}-skeleton.md      ❓过程/结果：骨架（见开放问题①）
volumes/{arc}/chapters/{ch}.md               结果产物：正文
```

### SSOT 文件格式（内容契约）

格式由对应 SPEC 定义（模型按此产出、程序按此切片/校验）。要点：

| 文件 | 关键结构 |
|---|---|
| 抽取骨架 `materials/{book}/skeleton.md` | 分块（~5万字/块）→ 情节节拍 · 爽点与钩子标注 · 人物出场 · 章末悬念类型（节奏统计入 DB，不塞此文件） |
| `creative-brief.md` | 值得借鉴的爽点系统/升级结构/节奏模型，逐项注明出处与理由（客观） |
| `creative-vision.md` | 题材定位 · 主角设定 · 金手指 · 预期规模（主观决策，不重复 brief） |
| 圣经九件套 `bible/*.md` | 世界观规则 · 爽点系统 · 金手指 · 主角弧度 · 人物档案 · 时间线 · 文风(style) · 禁忌 · 伏笔账本（标准 schema 见开放问题④） |
| 大纲 `outline/*.md` | 全书大纲 + 各卷大纲，只展开不创新 |
| 章节计划 `plan.md`（逐章条目） | `story_scope` · `protagonist_goal` · `obstacle` · `chapter_turn` · `reader_payoff` · `key_characters_state`（纯内容，**无状态字段**——状态在 DB） |
| 骨架 `{ch}-skeleton.md` | ≤1000 字：场景序列 · 场景时序与因果 · 伏笔处理 · 结尾钩子 |
| 正文 `{ch}.md` | 完整章节，严格依从骨架 |

### 切片约束（与 `engine.md` 的装配器耦合）

段级切片（如「圣经 style.md 里骨架相关的那一段」）要求 SSOT 文件有**机器可寻址的稳定结构**（命名小标题锚点），否则程序只能整文件塞入。

- 原则：**只在「高频被读 + 体量大」的产物上加结构**——圣经（几乎每步都读）用命名段落；大纲按卷/章加锚点。正文除「前一章整篇」外基本不作输入切入，不用结构化。
- 见开放问题②。

## DB —— 机器变量 + 用户配置（项目内本地 SQLite）

> 列名 `snake_case`，schema 定稿后确定。以下为设计草案。

| 表 | 存什么 | 作用 |
|---|---|---|
| `project_state` | phase、当前卷、各游标、时间戳 | 状态机的输入 |
| `chapter` | id、arc、status、verify_flag、deviation、骨架/正文文件 hash | 章节流水线 |
| `batch` | id、arc、章节集、status | 调度单位 |
| `artifact_ref` | path、kind(spec/过程/结果)、content_hash、version | **DB↔SSOT 的桥**（靠 hash 认变更） |
| `generation_log` | step、spec 路径+hash、本次读的输入切片及其 hash、token 进/出、成本 | 可复现 + AP-002 的 token 对账 |
| `bible_version` | 版本、原因、触发源(void/人工)、时间 | 圣经版本化 |
| `void_record` | 章节、原因、影响范围、时间 | VOID 记录 |
| `decision_log` | 关卡、判定(CONFIRMED/REVISE/VOID)、人写的评论/补充信息、时间 | 人的决定入库 |
| `check_result` | 章节/批、检查项、指标值、通过与否 | 客观红线结果 |
| `stat` | 对话占比、章字数、钩子密度等节奏统计 | 参考书风格画像 + 写作对账 |
| `config` | 红线阈值等（用户可改） | 「给用户打开的系统配置」 |

### DB↔SSOT 的桥：hash 游标

`artifact_ref.content_hash` + `generation_log` 里记录的输入切片 hash，共同支撑「过期检测」（见 `domain-model.md` 的 VOID 机制）。product.md 里已有的 `product_md_hash` 游标就是这个思路的先例。

## 开放问题

1. **过程/结果产物的判据**：判据是「读者/作者最终要的东西」(→ 结果) vs「为产出它而搭的脚手架」(→ 过程)，还是别的？据此**圣经**、**骨架**各归哪一类？
2. **能否接受圣经/大纲遵守一套轻量小标题锚点约定**（切片可行性的前提）。
3. **节奏统计入 DB**（`stat` 表，不塞 SSOT 的 json）—— 待确认。
4. 圣经九件套的标准 schema：哪些字段结构化供程序校验、哪些自由文本供模型阅读？（#6 开放问题①）
