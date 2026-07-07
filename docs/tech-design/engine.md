# Tech Design · Engine

> 执行器：项目文件布局、部署、SDK 调用、切片装配、人机决定接口、客观检查器（设计草案，planned）。核心循环见 `overview.md`；存储落地见 `storage.md`。

## 项目文件布局（运行时全景）

`mojian new <项目>` 后，一个项目目录同时容纳三层：

```
{项目}/
  # —— SPEC（部署副本，claude 原生读）——
  .claude/agents/*.md        SOP agent SPEC
  .claude/skills/            skills
  CLAUDE.md                  项目级指令
  prompts/                   写作指南等 agent 会 Read 的资产

  # —— SSOT（创作内容，人直读直改）——
  references/book/*.txt      输入：参考小说原文
  materials/ creative/ bible/ outline/ volumes/   过程/结果产物（布局见 storage.md）

  # —— DB（机器状态）——
  .mojian/state.db           本地 SQLite（状态机 · 统计 · 日志 · 配置）
```

**部署 = SPEC 怎么到模型手里**：主副本在客户端，`mojian new` 拷进项目的 `.claude/agents` 等（见 `storage.md`）。之后在项目目录里跑 `claude` 就能原生读到 agent。

**Rust 仍拥有状态机**：部署只解决「SPEC 落哪」；「什么时候跑、跑哪个 agent、喂哪些切片、能写哪些文件」由 Rust 状态机决定，零 token。二者不冲突——这保住 AP-001（状态机是代码不是散文）。

## SDK 调用：Rust → 无头 `claude` 子进程

没有原生 Rust 版 Agent SDK。Rust 在**项目目录内**把 `claude` 当子进程跑无头模式，引用**已部署的 agent** + 传参数，而非把整段提示词内联：

```
claude -p "用 skeleton-agent 处理 CH-001..003，参数如下：<切片参数>"
        --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope>
```

- claude 读 `.claude/agents/skeleton-agent.md`（部署的 SPEC）+ Rust 传的参数 + 白名单内 SSOT 文件。
- Rust 解析 JSON（结果 / 成本 / usage）落 `generation_log`。
- 备选（不采用）：Rust 直接打 Anthropic API —— 丢了工具循环，得自己实现 Read/Write。

## bundle：一次 SDK 调用喂进去的东西

| 字段 | 内容 | 举例（生成一批骨架） |
|---|---|---|
| `agent` | 引用的部署 agent | `.claude/agents/skeleton-agent.md` |
| `spec_slice` | SPEC 里跟本步相关的切片 | `style.md` 中骨架相关的几条约束 |
| `inputs` | 切片后的 SSOT，作结构化参数下发 | 本批章节计划 + 卷大纲切片 + 圣经切片(rules/protagonist) + 前一章骨架 + 活跃失败模式 + 到期伏笔 |
| `write_scope` | **允许写的文件白名单** | 只这几章的结果产物文件 |
| `output_contract` | 期望产出 + done 信号形状 | 骨架写入指定文件 + 字数统计回传 |

**最小执行上下文 = 切片后的 SPEC + 切片后的 SSOT + 前情，不是全量。** 由 Rust 按当前状态算出，而非让 AI 自己挑文件读（AP-002 的正面解法）。

## 切片装配器 + 输入契约 manifest

每个 SOP 步骤在其 SPEC 里声明**输入契约**（manifest），用**符号引用**而非死路径：

```yaml
# skeleton agent 的输入契约（写在 agent frontmatter 或 sidecar）
inputs:
  - bible.rules                 # 整文件
  - bible.style#skeleton        # style.md 里名为 skeleton 的那一段（段级切片）
  - bible.protagonist
  - outline.arc:{arc_id}
  - plan.chapters:{batch}       # 本批章节计划
  - prev_skeleton:{ch-1}
  - failure_pattern.active
  - foreshadowing.due:{arc_id,batch}
write:
  - chapter.skeleton:{batch}
```

- Rust 装配器把 `{arc_id}` `{batch}` 等符号按当前 DB 状态解析成具体路径 + 段落抽取（`#anchor`）+ 内容 hash。
- `write:` 声明喂给 `bundle.write_scope`（沙箱写白名单）。
- `#anchor` 段级切片依赖 SSOT 稳定小标题锚点（见 `storage.md` 开放问题②）。
- 「参数优先、缺失回退读文件」：切片作参数内联；部署 agent 仍可 Read 白名单内文件作兜底（对齐 ink_node 双路径，保证中断恢复健壮）。

## 人机决定接口（CLI）

不是光秃秃的 `next` 自动流转——很多转移要人给决定 + 补充信息才动。

```
mojian status                     # 当前状态 + 待办 + 「卡在等什么」
mojian run                        # 执行下一个「非人工」动作（装配+调SDK+检查），跑到关卡就停
mojian decide <关卡> <判定> [--comment "..." | --file ...]
        #   CONFIRMED
        #   REVISE CH-003 --comment "钩子太弱，结尾改悬念式"
        #   VOID   CH-002 --comment "节奏问题"
        #   补 story_scope（缺失时程序停机等人补）
```

节奏：`run` 跑到撞关卡 → 人 `decide`（带评论/补充）→ 再 `run`。**人的评论既落 `decision_log`，又被装配器切进下一次生成的 bundle**（对应 ink_node 里 agent 读 human-review.md 历史评论；mojian 改为结构化 CLI + DB，可对账可追溯）。

> 命令面（子命令全集）为 planned，schema 定稿后确定。

## 客观检查器（零 token）

Rust 实现，在**进人工队列之前**跑：字数区间、对话占比、最长叙述段、与计划偏差率。超线**直接打回 `prose_drafting`，不进人工队列**——人只看「爽不爽」（AP-003 / AP-004 落地）。

- 建议用 Rust 重写 ink_node 的 `text-stats.py / skeleton-stats.py / plan-structure-check.py`（零外部依赖、确定性）。
- 结果落 `check_result` 表。

## 两个「确定性 > 纪律」的升级

1. **圣经神圣 = 沙箱强约束**：写作步骤子进程 `write_scope` 白名单里根本没有 `bible/`，物理上写不了。ink_node 靠 SPEC 一条「禁止修改 bible/」纪律，mojian 靠沙箱。
2. **红线在人工队列之前**：见上。规则违反从「靠验证抓」变成「物理上做不到」/「自动打回」。

## 开放问题

1. **红线默认值**：直接用实测经验值（约 2300-3000 字/章、对话占比 15-50%）+ 留「首卷后重定标」钩子？还是别的？（#7 开放问题② / `infra.md`）
2. `claude` 无头模式的具体参数面（allowedTools 粒度、`--add-dir` sandbox 约束、成本/usage 字段解析）需实测一次调用后固化。
3. 输入契约 manifest 放 agent frontmatter 还是 sidecar 文件？符号引用语法（`#anchor` / `:{param}`）需与 SSOT 锚点约定一起定稿。
