# Product

> mojian = SPEC（提示词资产）+ 确定性执行器（Rust）。理念与架构见 `docs/tech-design/`。本文记录**已落地**的用户可见能力（What）。

## 已落地功能

### 运行环境就位（ITER-001，里程碑）

mojian 执行器骨架落地，作者可以建出并检视一个「运行环境就位」的小说项目：

- **`mojian new <dir>`**：建项目目录、在客户端中央 DB 登记项目（分配 `project_id`）、把 SPEC 主副本部署进项目、写出 `mojian.toml`；初始 SOP 阶段为 `style_sampling`。
- **`mojian status [--path <dir>]`**：读回客户端中央 DB，输出该项目当前的 SOP 阶段；打开时按 hash 比对，SPEC 部署漂移会被自动覆盖还原。
- **`mojian run` / `mojian decide`**：ITER-001 阶段为桩；已于 ITER-002 转为真逻辑（见下节「生成闭环」）。

客户端中央 SQLite（`central.db`）承载全部机器状态（12 表 + `schema_meta` 迁移）；项目目录内只有 SPEC 部署缓存与 `mojian.toml`，不含机器状态。SPEC 本迭代为**占位骨架**，仅验证「部署 + hash 覆盖」通路，真实提示词待 SOP #5/#6/#7 定稿后填充。

### 生成闭环（ITER-002，里程碑）

`run` / `decide` 从桩转为真逻辑，打通「生成闭环」的机制通路——装配切片 → 调（可替换的）生成命令 → 撞人工关卡即停 → 人给决定后推进 / 回喂：

- **`mojian run [--path <dir>]`**：定位项目 → 打开时按 hash 覆盖还原 SPEC → 循环状态机 `next_action`；到 Generate 步就按输入契约 manifest 装配最小切片 bundle（SPEC 切片 + SSOT 切片 + 前情），调无头 `claude` 子进程生成、把结果与 token/成本追加进 `generation.jsonl`，然后停在人工关卡；无可跑动作时正常退出。
- **`mojian decide <关卡> <判定> [目标] [--comment "..." | --file <path>]`**：三判定 `CONFIRMED` / `REVISE` / `VOID`。评论既追加进 `decision.jsonl`，又被下一次装配切进 bundle 回喂给模型。`CONFIRMED` 清关卡并推进；`REVISE` 回退对应细粒度状态；`VOID <CH>` 记 `void_record` 且章节 `void → planned`（最小语义，不级联 / 不过期检测）。
- **`mojian status [--path <dir>]`**：在 SOP 阶段之外，追加显示「卡在哪个关卡 / 等待哪种判定」。

生成命令默认 `claude`，可经 `MOJIAN_CLAUDE_CMD` 环境变量注入替换为假命令（测试端到端跑通 `run → decide → run` 而不触达真实 claude、不花 token）。机器状态与 `generation.jsonl` / `decision.jsonl` 日志落客户端（按 `project_id`），项目目录不含机器状态。本迭代仍用占位 SPEC（一个可跑的 `brief_drafting` 步 + `brief` 关卡）只验机制通路，真实创作提示词与多关卡链路待后续迭代。

## 尚未落地

创作生成循环的机制通路已于 ITER-002 落地（见上）。尚未落地：客观检查器（字数 / 对话占比 / 结构 + `check.jsonl`）、VOID 圣经级联与输入过期检测、真实 `claude --output-format json` schema 收口（本迭代只跑 mock）、真实 SOP 提示词与多关卡链路（当前为占位 SPEC）、题材 SPEC 变体、统计产出——见 `docs/tech-design/engine.md`。
