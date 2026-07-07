# Tech Design · Overview

> 本篇是 mojian 执行器的**项目级技术设计总纲**（理念定型阶段的设计草案，多数为 `planned`，未由代码证明的部分标注 `❓待定` / `TODO`，不作为已实现事实）。文件夹索引与阅读顺序见 `docs/tech-design.md`（WIKI）。总纲 RFC 见 GitHub #1，三个 SOP 设计见 #5/#6/#7。

## 一句话架构

**mojian = SPEC（提示词资产）+ 确定性执行器（Rust）。**

生成链：

```
SPEC + 输入      → 过程产物
SPEC + 过程产物  → 结果产物
```

程序（DB + 状态机）只管两件事：**现在该跑哪个 SPEC、喂它哪些切片**。创意与品味归模型和人，程序不碰。

## 三层数据边界（名词定义，全项目统一，不得混叫）

| 层 | 是什么 | 谁生成 / 谁改 | 落盘位置 |
|---|---|---|---|
| **SPEC** | 所有提示词 + 规则。每个 SOP 步骤「读什么、干什么、输出什么、自检闸门、禁止项、VOID 怎么走」都写在 SPEC 里 | 框架/贡献者写；**主副本在 mojian 客户端，新建项目时部署进项目**（详见 `storage.md`）；项目级可覆盖 | 主副本：客户端；部署副本：项目 `.claude/agents` 等 |
| **SSOT** | 人要直接读改的创作内容，分两半：**过程产物**（抽取信息、风格说明、圣经…）与**结果产物**（大纲、正文…） | SPEC + 输入/过程产物 → 生成；人可直改 | 创作内容 md 文件 |
| **DB** | 机器变量 + 给用户开放的系统配置。**不是创作内容** | 程序写，少量配置人改 | 本地 SQLite |

> 过程产物 / 结果产物的判据，以及圣经、骨架各归哪一类 —— 见 `storage.md` 开放问题①。

## 核心循环（执行器的心脏）

token 只花在其中一步（`SDK.跑`），其余全是免费的确定性 Rust：

```
loop {
  state   = DB.读状态()               // phase + 内部状态
  action  = 状态机.下一步(state)        // 纯函数，零 token —— 压 AP-001
  if action.要人 { 停，等人 decide }    // 到关卡了
  bundle  = 装配器.切片(action, state)  // 算出最小 SPEC+SSOT 切片 —— 压 AP-002
  result  = SDK.跑(bundle)             // claude -p 无头子进程，唯一花 token 的一步
  checks  = 检查器.客观校验(result)      // 字数/对话占比/结构，零 token —— 压 AP-003
  DB.落库(action, result, checks)
}
```

细节：状态机见 `domain-model.md`；部署、切片、SDK 调用、检查器见 `engine.md`。
