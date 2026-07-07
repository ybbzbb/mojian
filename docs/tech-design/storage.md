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
  bible/*.md                                   过程产物：圣经九件套
  outline/*.md                                 结果产物：大纲
  volumes/{arc}/plan.md                        过程产物：章节计划（内容）
  volumes/{arc}/chapters/{ch}-skeleton.md      过程产物：骨架
  volumes/{arc}/chapters/{ch}.md               结果产物：正文
  mojian.toml                                  身份标记：project_id（+ 已部署 SPEC 版本）；非机器状态
```

项目里**没有 DB**——机器状态全在客户端 `central.db`，靠 `mojian.toml` 的 `project_id` 关联。

**过程/结果判据**：结果产物 = 作者/读者最终交付物（大纲、正文）；过程产物 = 为产出它们而搭的设定与脚手架（抽取信息、风格说明、圣经、章节计划、骨架）。

### SSOT 文件格式（内容契约，由 SPEC 定义）

| 文件 | 关键结构 |
|---|---|
| 抽取骨架 `materials/{book}/skeleton.md` | 分块（~5万字/块）→ 情节节拍 · 爽点与钩子标注 · 人物出场 · 章末悬念类型（节奏统计入 DB，不塞此文件） |
| `creative-brief.md` | 值得借鉴的爽点系统/升级结构/节奏模型，逐项注明出处与理由（客观） |
| `creative-vision.md` | 题材定位 · 主角设定 · 金手指 · 预期规模（主观决策，不重复 brief） |
| 圣经九件套 `bible/*.md` | 世界观规则 · 爽点系统 · 金手指 · 主角弧度 · 人物档案 · 时间线 · 文风(style) · 禁忌 · 伏笔账本（结构化 schema 属 SOP② 设计，见 #6） |
| 大纲 `outline/*.md` | 全书大纲 + 各卷大纲，只展开不创新 |
| 章节计划 `plan.md`（逐章条目） | `story_scope` · `protagonist_goal` · `obstacle` · `chapter_turn` · `reader_payoff` · `key_characters_state`（纯内容，**无状态字段**——状态在客户端 DB） |
| 骨架 `{ch}-skeleton.md` | ≤1000 字：场景序列 · 场景时序与因果 · 伏笔处理 · 结尾钩子 |
| 正文 `{ch}.md` | 完整章节，严格依从骨架 |

### 切片约束（与 `engine.md` 装配器耦合）

段级切片（如「圣经 style.md 里骨架相关的那一段」）要求 SSOT 文件有**机器可寻址的稳定结构**（命名小标题锚点），否则只能整文件塞入。约定：**圣经、大纲采用轻量小标题锚点**（供程序段级切片）——只在「高频被读 + 体量大」的产物上加结构；正文除「前一章整篇」外不作输入切入。

## 四、DB 表设计（客户端 `central.db`）

**状态入 DB，日志入文件。** 当前/可查询的状态放 SQLite；只增不改的历史流（生成/决定/检查）放文件（见「五、日志」）。

### 对象关系图

```
                        ┌──────────────┐
                        │   project    │  项目登记（中央唯一）
                        └──────┬───────┘
        ┌──────────────┬───────┼───────────┬──────────────┬────────────┐
        │1:1           │1:N    │1:N        │1:N           │1:N         │1:N
        ▼              ▼       ▼           ▼              ▼            ▼
┌───────────────┐ ┌────────┐ ┌────────┐ ┌───────────┐ ┌──────────┐ ┌────────┐
│ project_state │ │reference│ │ volume │ │bible_     │ │artifact_ │ │ config │
│  (phase/游标) │ │ _book  │ │(卷/Arc)│ │ version   │ │ ref      │ │(覆盖项)│
└───────────────┘ └────────┘ └───┬────┘ └───────────┘ └──────────┘ └────────┘
                                  │1:N ┌───────────┐
                                  ├────┤   batch   │
                                  │    └─────┬─────┘
                                  │1:N       │1:N
                                  ▼          ▼
                              ┌───────────────────┐      ┌─────────────┐
                              │      chapter      │◄─1:N─┤ void_record │
                              │ (最细状态机载体)  │      └─────────────┘
                              └───────────────────┘
                              ┌───────────────────┐
                              │       stat        │  节奏统计（book/chapter 维度）
                              └───────────────────┘
```

### 建表（`central.db`；列名 `snake_case`，SQLite 类型）

```sql
CREATE TABLE project (
  project_id    TEXT PRIMARY KEY,          -- UUID
  name          TEXT NOT NULL,
  path          TEXT NOT NULL,             -- 项目目录绝对路径
  spec_version  TEXT,                      -- 已部署的 SPEC 版本
  spec_hash     TEXT,                      -- 部署缓存校验用
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE project_state (               -- 1:1 project
  project_id       TEXT PRIMARY KEY REFERENCES project(project_id),
  sop_phase        TEXT NOT NULL,          -- style_sampling … bible_building … writing
  current_volume   TEXT REFERENCES volume(id),
  cursors          TEXT,                   -- JSON：跨步骤游标（如抽取块游标聚合视图）
  updated_at       TEXT NOT NULL
);

CREATE TABLE reference_book (              -- SOP① 参考小说
  id             TEXT PRIMARY KEY,
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  title          TEXT NOT NULL,
  extract_status TEXT NOT NULL,            -- pending | extracting | extracted
  block_cursor   INTEGER NOT NULL DEFAULT 0,  -- 当前 ~5万字块索引
  updated_at     TEXT NOT NULL
);

CREATE TABLE volume (                      -- 卷 / Arc
  id                TEXT PRIMARY KEY,       -- ARC-xxx
  project_id        TEXT NOT NULL REFERENCES project(project_id),
  seq               INTEGER NOT NULL,
  name              TEXT,
  arc_phase         TEXT NOT NULL,          -- arc_planning | arc_plan_review | writing | arc_done
  chapters_total    INTEGER NOT NULL DEFAULT 0,
  chapters_approved INTEGER NOT NULL DEFAULT 0,
  deviation         INTEGER NOT NULL DEFAULT 0,  -- 实际章数 - 大纲预期
  updated_at        TEXT NOT NULL
);

CREATE TABLE batch (                       -- 调度单位（每批 3-5 章）
  id          TEXT PRIMARY KEY,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  volume_id   TEXT NOT NULL REFERENCES volume(id),
  status      TEXT NOT NULL,
  created_at  TEXT NOT NULL
);

CREATE TABLE chapter (                     -- 最细状态机载体
  id             TEXT PRIMARY KEY,          -- CH-xxx
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  volume_id      TEXT NOT NULL REFERENCES volume(id),
  batch_id       TEXT REFERENCES batch(id), -- 未入批为 NULL
  seq            INTEGER NOT NULL,
  status         TEXT NOT NULL,             -- planned | skeleton_drafting | skeleton_review | prose_drafting | prose_review | approved | void
  verify_flag    TEXT,                      -- clean | suspect
  skeleton_path  TEXT,  skeleton_hash TEXT,
  prose_path     TEXT,  prose_hash    TEXT,
  updated_at     TEXT NOT NULL
);

CREATE TABLE artifact_ref (                -- DB↔SSOT 的桥
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id   TEXT NOT NULL REFERENCES project(project_id),
  path         TEXT NOT NULL,
  kind         TEXT NOT NULL,              -- input | process | result
  content_hash TEXT NOT NULL,              -- 认变更、驱动过期检测
  version      INTEGER NOT NULL DEFAULT 1,
  updated_at   TEXT NOT NULL
);

CREATE TABLE bible_version (               -- 圣经版本化
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  version     INTEGER NOT NULL,
  reason      TEXT,
  trigger     TEXT NOT NULL,               -- void | human
  created_at  TEXT NOT NULL
);

CREATE TABLE void_record (                 -- 作废记录
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id     TEXT NOT NULL REFERENCES project(project_id),
  chapter_id     TEXT NOT NULL REFERENCES chapter(id),
  reason         TEXT,
  affected_scope TEXT,                      -- JSON：受影响章节 id 列表
  created_at     TEXT NOT NULL
);

CREATE TABLE stat (                        -- 节奏统计
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  scope       TEXT NOT NULL,               -- book:{id} | chapter:{id}
  metric      TEXT NOT NULL,               -- dialogue_ratio | hanzi | hook_density | …
  value       REAL NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE config (                      -- 项目配置覆盖项（默认在 defaults.toml）
  project_id  TEXT NOT NULL REFERENCES project(project_id),
  key         TEXT NOT NULL,
  value       TEXT NOT NULL,
  PRIMARY KEY (project_id, key)
);

CREATE TABLE schema_meta (                 -- 迁移用
  schema_version INTEGER NOT NULL
);
```

### DB↔SSOT 的桥 & 升级

- **过期检测**：`artifact_ref.content_hash` + 生成日志里记录的输入切片 hash，共同支撑「圣经改了 → 哪些章节输入过期」的免费查询（见 `domain-model.md` VOID 机制）。
- **升级**：`schema_meta.schema_version` 驱动迁移；程序升级时**只在客户端中央库跑一次**，不逐项目处理。

## 五、日志（文件，非 DB）

只增不改的历史流用文件，人可直接 tail、不膨胀 DB。落在**客户端**数据目录、按 `project_id` 分目录（仍属程序所有，与 SSOT 无关）：

```
mojian 客户端数据目录/logs/{project_id}/
  generation.jsonl    每次 SDK 生成：step · agent · spec 路径+hash · 输入切片及其 hash · token 进/出 · 成本 · 时间
  decision.jsonl      人在关卡的决定：关卡 · 判定(CONFIRMED/REVISE/VOID) · 目标(章/批) · 评论/补充 · 时间
  check.jsonl         客观检查：目标 · 检查项 · 指标值 · 通过与否 · 时间
```

- 格式 JSONL（一行一事件），程序追加写。
- `decision.jsonl` 的评论被装配器切进下一次生成的 bundle（对应 ink_node 的 human-review.md）。
- `generation.jsonl` 里的输入切片 hash 供过期检测与 token 对账（AP-002）。
