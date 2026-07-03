# Tech Design

> 本项目处于理念定型阶段，代码尚未落地。以下为**已确定的项目级技术约束与目标架构**；未由代码证明的部分标注 `TODO`（planned），不作为已实现事实。

## System Architecture

单一 Cargo workspace，本地单机执行。分层：**提示词资产（SOP）+ 确定性执行器（Rust）**。

```
mojian/（Cargo workspace）
├── prompts/               # ★ 核心资产：三个 SOP 的提示词包 + 写作指南（明文、运行时加载、可覆盖）
│   ├── sop-1-style/       #   风格抽取
│   ├── sop-2-bible/       #   圣经·大纲构建
│   └── sop-3-writing/     #   写作循环
├── crates/mojian-core/    # 库：状态机 · SQLite 存储 · 文本统计 · 上下文装配
├── crates/mojian-cli/     # 二进制 `mojian`：SOP 调度接口（next/claim/done/…）
└── app/                   # 桌面客户端（Tauri 2）：审阅台 + Agent SDK 生成（后期里程碑）
```

**数据归属边界**（判断标准：人要不要直接读和改）：
- **机器状态**（phase、统计、日志、判定、游标、版本）→ SQLite
- **创作内容**（圣经、大纲、骨架、正文、评审意见）→ markdown 文件（人可直读直改）

## Module Responsibilities

| 模块 | 职责 | 状态 |
|------|------|------|
| `prompts/sop-1-style` | SOP① 风格抽取的提示词与写作指南 | 设计中（#5） |
| `prompts/sop-2-bible` | SOP② 圣经·大纲构建的提示词 | 设计中（#6） |
| `prompts/sop-3-writing` | SOP③ 写作循环的提示词 | 设计中（#7） |
| `crates/mojian-core` | 状态机、SQLite 存储、文本统计、上下文最小集装配、客观一致性检查 | TODO（planned） |
| `crates/mojian-cli` | 二进制 `mojian`，SOP 调度命令面（next/claim/done/…） | TODO（planned） |
| `app/` | Tauri 2 桌面客户端：审阅台、Agent SDK 生成 | TODO（后期） |

## Tech Stack

| 层 | 技术 | 版本 | 状态 |
|----|------|------|------|
| 语言 | Rust | TODO（拟 stable / edition 2021+） | planned |
| 工程组织 | Cargo workspace | — | planned |
| 存储 | SQLite | TODO（拟经 `rusqlite` / `sqlx`，待选型） | planned |
| CLI | Rust 二进制 `mojian` | — | planned |
| 桌面端 | Tauri | 2 | 后期 |
| 分发 | GitHub Release 起步；后续 Homebrew tap；`mojian-core` 发布至 crates.io | — | planned |

## Architectural Constraints

- **确定性归代码，创意归模型，品味归人**：状态转移、调度、统计、客观一致性检查（字数/对话占比/结构/时间线）必须由 Rust 代码实现，零 token；不得退化为「让 LLM 读规则后自律」。
- **数据归属不可混淆**：机器状态只进 SQLite，创作内容只留 markdown；两者不互相冗余存储。
- **演绎链条严格单向**：圣经 → 大纲 → 章节计划 → 骨架 → 正文，每层严格依从上层、只展开不创新。发现违反上层不改下层迁就，而是走 VOID（作废 → 修上层 → 重写）。
- **骨架先行**：正文渲染前必须先产出 ≤1000 字骨架并通过人工关卡，不得跳过直接写正文。
- **提示词是运行时资产**：SOP 提示词明文存于 `prompts/`，运行时加载，支持项目级覆盖；不得把提示词硬编码进 Rust 源码。
- **上下文最小集**：由程序计算每步该读的最小文件集，禁止「每个角色把所有文件再读一遍」的重复装载。

## Adding New Dependencies

Rust crate 依赖新增需经人工确认，并在对应迭代的 `tech-design.md` 中声明选型理由。关键选型（SQLite 驱动、异步运行时、Tauri 版本）属项目级决策，须在本文件更新后方可引入。禁止 builder-agent 自行引入本文件未声明的重量级依赖。
