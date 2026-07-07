# Tech Design（WIKI）

> `docs/tech-design/` 文件夹的入口与导航。mojian 执行器的项目级技术设计，处于理念定型阶段（设计草案，`planned`；未由代码证明的部分标注 `❓待定` / `TODO`）。

## 一句话架构

**mojian = SPEC（提示词资产）+ 确定性执行器（Rust）。**

生成链：`SPEC + 输入 → 过程产物`；`SPEC + 过程产物 → 结果产物`。程序（DB + 状态机）只管「现在该跑哪个 SPEC、喂它哪些切片」；创意与品味归模型和人。

## 三层数据边界（名词，全项目统一）

| 层 | 是什么 | 落盘位置 |
|---|---|---|
| **SPEC** | 所有提示词 + 规则 | 主副本在客户端，部署副本在项目 `.claude/agents` 等 |
| **SSOT** | 人直读直改的创作内容：过程产物 + 结果产物 | 项目内创作 md 文件 |
| **DB** | 机器变量 + 用户配置（非创作内容） | 项目内本地 SQLite |

## 阅读顺序

| 篇 | 管什么 |
|---|---|
| [overview.md](tech-design/overview.md) | 总纲：分层、三层边界、核心循环 |
| [domain-model.md](tech-design/domain-model.md) | 领域实体 + 两级状态机 + VOID |
| [storage.md](tech-design/storage.md) | SPEC（主副本→部署）/ SSOT（文件格式）/ DB 的落地 |
| [engine.md](tech-design/engine.md) | 执行器：项目布局、部署、SDK 调用、切片装配、人机决定、检查器 |

## 开放问题（汇总，待收敛）

1. **过程/结果产物的判据** + 圣经、骨架各归哪一类（storage.md）
2. **圣经/大纲的轻量小标题锚点约定**——切片可行性前提（storage.md）
3. **节奏统计入 DB** 确认（storage.md）
4. **红线默认值**：经验值 + 首卷重定标钩子，还是别的（engine.md）

## 相关

- 总纲 RFC：GitHub #1
- 三个 SOP 设计：#5（风格抽取）/ #6（圣经·大纲）/ #7（写作循环）
- 前身协议参考实现：`ink_node`（16 agent 的 markdown 多 Agent 创作协议）
