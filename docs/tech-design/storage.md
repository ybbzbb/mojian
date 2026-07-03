# Tech Design · Storage

> 三层存储（SPEC / SSOT / DB）的落地（设计草案，planned）。三层定义见 `overview.md`。

## SPEC —— `prompts/`（+ guides）

版本化提示词资产。

- **覆盖链**：项目本地 `prompts/` 覆盖框架自带默认（详见 `prompts.md`）。
- 运行时加载，**不进 DB**。
- 每次生成时，DB 的 `generation_log` 记下用了哪个 SPEC 的路径 + hash（为可复现）。

## SSOT —— 创作内容文件（人直读直改）

目标布局（planned；具体目录名待首个实现迭代确认）：

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

### 切片约束（与 `engine.md` 的装配器耦合）

段级切片（如「圣经 style.md 里骨架相关的那一段」）要求 SSOT 文件有**机器可寻址的稳定结构**（命名小标题锚点），否则程序只能整文件塞入。

- 原则：**只在「高频被读 + 体量大」的产物上加结构**——圣经（几乎每步都读）用命名段落；大纲按卷/章加锚点。正文（结果产物）除「前一章整篇」外基本不作输入切入，不用结构化。
- 见开放问题②。

## DB —— 机器变量 + 用户配置（本地 SQLite）

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

`artifact_ref.content_hash` + `generation_log` 里记录的输入切片 hash，共同支撑「过期检测」（见 `domain-model.md` 的 VOID 机制）。product.md 里已有的 `product_md_hash` 游标就是这个思路的先例，统一推广。

## 开放问题

1. **过程/结果产物的判据**：判据到底是「读者/作者最终要的东西」(→ 结果) vs「为产出它而搭的脚手架」(→ 过程)，还是别的？据此**圣经**、**骨架**各归哪一类？（切片与归档需要这个准绳）
2. **能否接受圣经/大纲遵守一套轻量小标题锚点约定**（切片可行性的前提）。
3. **节奏统计入 DB**（`stat` 表，不塞 SSOT 的 json）—— 待确认。
4. 圣经九件套的标准 schema：哪些字段结构化供程序校验、哪些自由文本供模型阅读？（#6 开放问题①）
