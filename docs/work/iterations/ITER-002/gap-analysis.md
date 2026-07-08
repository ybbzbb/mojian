# GAP Analysis — ITER-002

date: 2026-07-07
revision: 1
source: issue#11,#12
status: confirmed

> Round 1 CONFIRMED 终审通过（2026-07-07）：3 项开放边界经人工裁决（客观检查器排除、VOID 不级联、验收=一次 `run→decide→run` 机制通路），已并入 requirements.md「已确认边界」。本 GAP 内容不变，status → confirmed。

## 需求摘要

把 ITER-001 留下的 `run` / `decide` 桩变成真逻辑，打通 mojian 的「生成闭环」：程序按 SPEC 输入契约算出某步的最小上下文切片、组装 bundle、以无头 `claude` 子进程执行一次生成并解析 JSON、把生成事件落进客户端 `generation.jsonl`（IMPL-3）；`mojian run` 让状态机推进「下一个非人工动作」直到撞人工关卡即停、`mojian decide` 在关卡给判定（CONFIRMED/REVISE/VOID + 补充）并写 `decision.jsonl` 推进状态机、人的评论被下一次装配切回 bundle（IMPL-4）。SPEC 当前仍为占位骨架——本迭代验证切片 / 调用 / 循环的**机制通路**，不追求真实创作产物。

## 功能 GAP

| 功能点 | 当前状态 | 目标状态 | 差距 |
|--------|---------|---------|------|
| 输入契约 manifest 解析 | 无 | 解析 SPEC 步骤声明的符号引用（如 `bible.style#skeleton`、`plan.chapters:{batch}`、`prev_skeleton:{ch-1}`）→ 具体路径 / 段落抽取 / 内容 hash | 全新（engine.md「切片装配器 + 输入契约 manifest」） |
| 上下文切片 | 无 | 段级切片（圣经/大纲 `#anchor` 锚点）+ 整文件切片；`{arc_id}`/`{batch}` 等符号按当前 DB 状态解析 | 全新 |
| bundle 组装 | 无 | 组装五字段：`agent` / `spec_slice` / `inputs` / `write_scope`（沙箱写白名单，从 manifest `write:` 推导）/ `output_contract` | 全新 |
| SDK 调用（无头 claude） | 桩（打印「stub」exit 0） | Rust 把 `claude` 当子进程跑无头模式（`-p` + `--output-format json` + `--allowedTools` + `--add-dir <write_scope>`），解析回 JSON（结果 / 成本 / usage） | 桩 → 真逻辑；须**外部命令可替换**以供测试注入假命令 |
| generation.jsonl 写入 | 无（ITER-001 只建日志目录约定） | 每次生成追加一行：step · agent · spec 路径+hash · 输入切片及其 hash · token 进/出 · 成本 · 时间；落客户端 `logs/{project_id}/` | 全新（storage.md「六、日志」） |
| `mojian run` | 桩 | 状态机算「下一个非人工动作」→ 装配 → 调 SDK → 落库/推进；撞人工关卡即停。「客观检查」步本迭代留位不实现（裁决①） | 桩 → 真逻辑 |
| `mojian status`（卡点显示） | 已实现（输出当前 SOP phase） | 追加：显示卡在哪个人工关卡 / 在等什么决定 | 增量扩展 |
| `mojian decide` | 桩 | `decide <关卡> <判定> [--comment "..." \| --file ...]`：CONFIRMED / REVISE CH-xxx / VOID CH-xxx + 补充信息 → 写 `decision.jsonl` → 推进状态机（VOID 仅最小语义，裁决③） | 桩 → 真逻辑 |
| decision 回喂 | 无 | 装配器把 `decision.jsonl` 里的人类评论切进下一次生成的 bundle（对应 ink_node 的 human-review.md 历史评论） | 全新 |

## 系统 / 架构 GAP

| 层级 | 当前状态 | 需要变更 | 影响范围 |
|------|---------|---------|---------|
| 数据模型 | 12 表已建（ITER-001）；`chapter`/`artifact_ref`/`project_state` 等静态 | 生成/决定推进时读写状态：章节 status 流转、`artifact_ref.content_hash` 更新、`project_state`/游标推进（本迭代按机制通路所需最小落库范围） | 客户端中央 DB（既有表，无需改 schema） |
| 日志（文件层） | 只有目录约定，无写入代码 | 新增追加写 `generation.jsonl` + `decision.jsonl`（JSONL、只增不改、落客户端 `logs/{project_id}/`） | 新增文件写入路径 |
| 切片装配 | 无 | 新增装配器模块：manifest 解析 + 符号解析 + 段级/整文件切片 + 内容 hash + bundle 组装 | `mojian-core` 新增核心模块，engine 的主承重 |
| SDK 调用层 | 桩 | 新增子进程调用抽象：默认真实 `claude`，可注入替身命令（可测试性硬约束）；JSON 解析 | `mojian-core` 新增；测试面关键 |
| 状态机推进 | 类型已定（`SopPhase`/`ChapterState`），无推进逻辑 | 新增「下一个非人工动作」纯函数 + 关卡判定 + `decide` 触发的转移 | `mojian-core` 状态机从类型走向行为 |
| CLI 接口 | `run`/`decide` 桩、`status` 基础实现 | `run`/`decide` 接真逻辑；`decide` 增参数面（关卡 + 判定 + `--comment`/`--file`）；`status` 增卡点显示 | `mojian-cli` 用户入口 |
| SPEC 输入契约声明 | 占位骨架无 manifest | 需在（占位）SPEC 步骤上有可被解析的输入契约 manifest（frontmatter 或 sidecar）供机制通路验证 | 跨 SPEC 主副本与装配器（占位程度由 design 定） |
| 前端模块 | N/A（纯 CLI） | 无 | — |

## 可行性结论

**结论：** ✅ 可直接实现（机制通路层面）；3 项范围边界已由人工在 Round 1 CONFIRMED 中裁决锁定（见 requirements.md「已确认边界」）。

- 项目级技术设计基线已把 SDK 调用形态（Rust → 无头 `claude` 子进程）、bundle 五字段、切片装配器 + 输入契约 manifest、人机决定接口、核心循环、三条 JSONL 日志格式都写清（engine.md + storage.md「六」+ overview.md），实现路径明确。
- 建立在 ITER-001 交付的稳固地基上（workspace、领域状态机类型、中央 DB 12 表、project 登记、SPEC 部署、CLI 骨架、28 测试全通过），本迭代是「把桩换成真逻辑」而非重新奠基。
- **可测试性有正面解法**：SDK 外部命令可替换（测试注入假命令）→ QA 无需真花 token 调 claude 即可验证装配 / 日志 / 循环全链路；真实 `claude` 为默认。
- 原先的范围不确定性（占位 SPEC 下 run 推进多深、客观检查器是否纳入、VOID 深度）已经人工裁决消解：客观检查器排除、VOID 不级联、验收 = 一次 `run→decide→run` 机制通路。

（不在此进行选项对比 — 切片抽取实现、子进程注入机制、manifest 承载形态等选型由 design-agent 在下一阶段输出。）

## product.md 变更建议

无。

`docs/product.md` 的「已落地功能 / 尚未落地」更新属归档阶段职责，由 archivist-agent 在迭代收口时按实际交付调整（届时把「创作生成循环」中已闭环的部分从「尚未落地」上移）。本需求阶段不改动。

## 明确排除

本次不在范围内（后续迭代）：

- **客观检查器的红线逻辑**（字数区间 / 对话占比 / 最长叙述段 / 与计划偏差率）与 `check.jsonl` 写入 —— 裁决① 确认排除本迭代，归 ITER-003（IMPL-5）；run 循环「客观检查」步留位不实现。
- **VOID 机制与过期检测**（圣经改动→受影响章节标记→按 SPEC void→planned）—— 裁决③ 确认排除本迭代，归 ITER-003（IMPL-6）。
- **题材 SPEC 变体**的客户端按项目管理。
- **stat 表数据产出**（节奏统计写入）。
- **真实提示词填充**（SPEC 仍为占位骨架，待 SOP #5/#6/#7 定稿）。
- 真实端到端创作产出（成品大纲 / 正文质量）——本迭代只验证机制通路。
- 多关卡完整链路验收 —— 裁决② 确认最小验收深度为一次 `run → decide → run`。
