# Requirements — ITER-002

date: 2026-07-07
revision: 2
status: confirmed

> revision 2 依据：Round 1 [human feedback] CONFIRMED + 3 项待确认项裁决，已并入下方「已确认边界」。终审通过，status → confirmed。

## 目标

打通 mojian 的「生成闭环」：把 ITER-001 留下的 `run` / `decide` 桩换成真逻辑。程序按 SPEC 输入契约算出某步的最小上下文切片、组装 bundle、以无头 `claude` 子进程执行一次生成并解析 JSON、把生成事件落进 `generation.jsonl`（IMPL-3）；`mojian run` 让状态机推进「下一个非人工动作」直到撞人工关卡即停、`mojian decide` 在关卡给判定（CONFIRMED/REVISE/VOID + 补充）写 `decision.jsonl` 并推进状态机、人的评论被下一次装配切回 bundle（IMPL-4）。SPEC 仍为占位骨架——本迭代验证切片 / 调用 / 循环的**机制通路**，真实提示词与真实创作产出不在目标内。

## 正式需求

### 来源：issue#11（IMPL-3 切片装配 + SDK 调用）

- REQ-001：程序能解析 SPEC 步骤声明的**输入契约 manifest**，把符号引用（如 `bible.style#skeleton`、`bible.rules`、`plan.chapters:{batch}`、`prev_skeleton:{ch-1}`、`foreshadowing.due:{arc_id,batch}` 等）按当前项目 DB 状态解析为**具体路径 / 段落抽取范围 / 内容 hash**。
- REQ-002：切片支持两种粒度——**段级切片**（依 SSOT 稳定小标题 `#anchor` 锚点抽取，如圣经/大纲的命名段落）与**整文件切片**；符号中的 `{arc_id}` / `{batch}` / `{ch-1}` 等占位按当前状态解析。
- REQ-003：程序能组装一次 SDK 调用的 **bundle**，含五字段：`agent`（引用的已部署 agent）、`spec_slice`（本步相关 SPEC 切片）、`inputs`（切片后的 SSOT 作结构化参数）、`write_scope`（允许写的文件白名单，由 manifest `write:` 声明推导）、`output_contract`（期望产出与 done 信号形状）。
- REQ-004：程序以**无头 `claude` 子进程**执行一次生成，调用形态对齐 engine.md：在项目目录内运行 `claude -p <参数> --output-format json --allowedTools Read,Write,Edit --add-dir <write_scope>`；`write_scope` 作为写沙箱白名单下发。
- REQ-005：程序解析子进程返回的 **JSON**，至少取出：生成结果、成本（cost）、usage（token 进/出）。
- REQ-006：每次生成向客户端 `logs/{project_id}/generation.jsonl` **追加一行**事件，字段至少含：step、agent、spec 路径 + hash、输入切片及其内容 hash、token 进/出、成本、时间戳。日志为 JSONL（一行一事件、只增不改、程序追加写）。

### 来源：issue#12（IMPL-4 决定循环）

- REQ-007：`mojian run` 让状态机计算「**下一个非人工动作**」并执行「装配 → 调 SDK → 落库/推进状态机」；当下一步是人工关卡时，`run` **停机**，不越过关卡。run 循环中的「**客观检查**」步骤本迭代**留位不实现**——生成后直接进关卡 / 落库（见「已确认边界 · 裁决①」）。
- REQ-008：`mojian status` 在项目卡于人工关卡时，显示**卡在哪个关卡 / 在等什么决定**（在 ITER-001 已有的「当前 SOP phase」输出基础上扩展）。
- REQ-009：`mojian decide <关卡> <判定> [--comment "..." | --file <path>]` 支持三种判定：`CONFIRMED`、`REVISE <目标 CH/批>`、`VOID <目标 CH/批>`，并可附带评论（`--comment`）或文件补充（`--file`）。
- REQ-010：`decide` 执行时向客户端 `logs/{project_id}/decision.jsonl` **追加一行**事件（字段至少含：关卡、判定、目标章/批、评论/补充、时间戳），并据判定**推进状态机**（CONFIRMED 放行、REVISE 打回对应细粒度状态、VOID 记作废并按「已确认边界 · 裁决③」的最小语义处理）。
- REQ-011：装配器在组装下一次生成的 bundle 时，把 `decision.jsonl` 中相关的**人类评论切进 `inputs`**（对应 ink_node 中 agent 读 human-review.md 历史评论），使 `decide --comment` 的评论出现在下一次生成的输入切片里。
- REQ-012：`run` → 撞关卡停 → `decide` → 再 `run` 的节奏可端到端跑通：`decide CONFIRMED` 后 `run` 能继续推进到下一动作/下一关卡（验收深度见「已确认边界 · 裁决②」）。

## 已确认边界（Round 1 CONFIRMED 裁决）

以下 3 项为人工在 Round 1 对开放待确认项的正式裁决，已作为本迭代**已确认边界**，与既定迭代划分（IMPL-5/IMPL-6 归 ITER-003）一致：

- **裁决①（客观检查器 + `check.jsonl` — 排除本迭代）**：客观检查器的红线逻辑（字数区间 / 对话占比 / 最长叙述段 / 与计划偏差率）与 `check.jsonl` 写入**排除本迭代**，属 IMPL-5，归 ITER-003。`run` 循环中「客观检查」这一步本迭代**留位不实现**——生成后直接进人工关卡 / 落库。
- **裁决②（机制通路最小验收深度 = 一次 `run → decide → run`）**：验收标准为**串通一次 `run → decide → run`**——`run` 执行「装配 → 调（可注入的 mock）SDK → 写 `generation.jsonl` → 推进到人工关卡即停」；`decide` 写 `decision.jsonl` → 推进状态机；人的评论被下一次 `run` 装配切进 bundle。用**注入的 mock SDK 命令端到端测**，**不要求真实创作产物**。不要求串起多个关卡（如 brief → vision）的完整链路。
- **裁决③（`decide VOID` 深度 — 最小语义）**：`decide VOID` 本迭代**仅记录 `void_record`（+ `decision.jsonl`）+ 最小状态推进**（按指令 `void → planned`）；**不做**圣经改动级联 / 输入 hash 过期检测——那是 IMPL-6，归 ITER-003。

## 约束

- **SDK 调用可测试**：生成调用的外部命令必须**可替换**（供测试注入假命令，无需真实花 token 调 `claude`），真实 `claude` 为默认。QA 须能在不触达真实 claude 的前提下验证装配 / 日志 / 决定循环全链路。
- **机器状态归属**：生成/决定产生的机器状态（章节 status、游标、artifact hash 等）与三条 JSONL 日志一律落客户端中央 DB / 客户端 `logs/{project_id}/`（按 `project_id` 分区）；项目目录内**不存机器状态**（沿用 ITER-001 与 overview.md 作用域约束）。项目内只被写入 SSOT 创作产物（在 `write_scope` 白名单内）。
- **沙箱写约束**：子进程只能写 `write_scope` 白名单内的文件；圣经等受保护路径不在写作步骤白名单内（物理上写不了，对齐 engine.md「圣经神圣 = 沙箱强约束」）。
- **命名对齐 `docs/naming.md`**：新增模块 / 函数 / 状态机 phase 名与三个 SOP 设计逐字对齐，代码与文档不得出现两套叫法。
- **SPEC 占位骨架**：本迭代 SPEC 仍为占位骨架，只验证切片 / 调用 / 循环的**机制通路**；真实提示词待 SOP #5/#6/#7 定稿后由后续迭代填充。为使 manifest 可被解析验证，占位 SPEC 需带最小可解析的输入契约声明（承载形态与占位程度由 design-agent 定）。
- 沿用 ITER-001 已锁定的技术栈基线（`rusqlite` bundled / `clap` v4 / `blake3` / `serde`+`toml` 等，见 overview.md「技术栈基线」）；引入子进程调用与 JSON 解析所需依赖由 design-agent 评估。

## 不在范围内

- **客观检查器**的红线逻辑（字数区间 / 对话占比 / 最长叙述段 / 与计划偏差率）与 `check.jsonl` 写入 —— 裁决① 确认排除本迭代，归 ITER-003（IMPL-5）；run 循环「客观检查」步本迭代留位不实现。
- **VOID 机制的完整语义与过期检测**（圣经改动 → 输入 hash 比对 → 标记受影响章节 → 按 SPEC 令 void→planned）——裁决③ 确认排除本迭代，归 ITER-003（IMPL-6）；`decide VOID` 本迭代只做「记录作废 + 最小状态推进 `void→planned`」。
- **题材 SPEC 变体**的客户端按项目管理。
- **stat 表数据产出**（节奏统计写入）。
- **真实提示词填充**与真实创作质量产出（成品大纲 / 正文）。
- **多关卡完整链路验收**（如 brief → vision 串联）——裁决② 确认最小验收深度为一次 `run → decide → run`。
- `docs/product.md` 内容更新（归档阶段处理）。

## 待确认项

- [x] 需求边界是否清晰？（本迭代 = 「生成闭环」机制通路：切片 + SDK 调用 + generation.jsonl + run/decide/decision.jsonl 回喂）— 已确认
- [x] 有无遗漏的关键场景？— 已确认，无遗漏
- [x] 约束是否合理？（尤其「SDK 外部命令可替换以供测试注入」这一可测试性硬约束）— 已确认
- [x] **① 客观检查器是否纳入本迭代？** — 裁决①：**排除本迭代**（归 ITER-003/IMPL-5），run 循环「客观检查」步留位不实现。
- [x] **② 占位 SPEC 下 `run` 的可推进深度？** — 裁决②：最小验收深度 = 串通一次 `run → decide → run`（可注入 mock SDK 端到端测，不要求真实创作产物，不要求多关卡链路）。
- [x] **③ `decide VOID` 的本迭代深度？** — 裁决③：仅 `void_record` + `decision.jsonl` + 最小状态推进 `void→planned`，不做圣经级联 / 过期检测（归 ITER-003/IMPL-6）。
