# Plan — ITER-002

updated: 2026-07-07

## Goal

把 ITER-001 留下的 `run` / `decide` 桩换成真逻辑，打通「生成闭环」的机制通路：输入契约 manifest 解析 + 双粒度切片 + 五字段 bundle 装配（IMPL-3）、无头 `claude` 子进程 SDK 调用 + JSON 解析、generation/decision 两条 JSONL 追加写、`run → decide → run` 状态机推进（IMPL-4）。占位 SPEC 只验机制，不产真实创作物；SDK 命令经 `MOJIAN_CLAUDE_CMD` / `GenerationRunner` trait 可注入 mock（硬约束）。

## Tasks

| Task | Title | Type | Status | Owner | Depends On |
|------|-------|------|--------|-------|------------|
| TASK-001 | workspace 依赖 serde_json + error 变体扩展 | backend | ready | builder-agent | — |
| TASK-002 | log 模块：generation/decision JSONL 追加写 + 评论回读 | backend | planned | builder-agent | TASK-001 |
| TASK-003 | sdk 模块：GenerationRunner trait + Bundle + ClaudeCliRunner | backend | planned | builder-agent | TASK-001 |
| TASK-004 | context 模块：manifest + 符号解析 + 切片 + assemble_bundle + 占位 SPEC 步 | backend | planned | builder-agent | TASK-002, TASK-003 |
| TASK-005 | engine + state 模块：next_action / apply_* + 运行时 DB 行读写 | backend | planned | builder-agent | TASK-002, TASK-003, TASK-004 |
| TASK-006 | CLI run/decide/status 收口 + run→decide→run 端到端 | backend | planned | builder-agent | TASK-005 |

依赖链（对齐 tech-design.md「涉及模块」自然依赖）：

```
TASK-001（workspace serde_json + error 变体）
   ├─→ TASK-002（log）──────────┐
   └─→ TASK-003（sdk）──────────┤
                                 ├─→ TASK-004（context: manifest/symbol/slice/assemble + 占位 SPEC 步）
                                 │        │
                                 └────────┴─→ TASK-005（engine + state）
                                                    │
                                                    └─→ TASK-006（CLI run/decide/status 端到端）
```

## 需求覆盖

| REQ | Tasks |
|-----|-------|
| REQ-001（manifest 符号解析） | TASK-004 |
| REQ-002（段级 / 整文件双粒度切片） | TASK-004 |
| REQ-003（五字段 bundle） | TASK-003（Bundle 类型定义）, TASK-004（assemble 组装） |
| REQ-004（无头 claude 子进程调用） | TASK-003 |
| REQ-005（JSON 解析 SdkResponse：result/cost/usage） | TASK-003 |
| REQ-006（generation.jsonl 追加写） | TASK-002（写入器）, TASK-005（run 中调用） |
| REQ-007（run 计算下一非人工动作 + 撞关卡停） | TASK-005（next_action/engine）, TASK-006（run.rs） |
| REQ-008（status 显卡点） | TASK-006 |
| REQ-009（decide 三判定 CLI 解析） | TASK-006（CLI 解析）, TASK-005（apply_decision 语义） |
| REQ-010（decision.jsonl + 推进状态机） | TASK-002（append_decision）, TASK-005（apply_decision）, TASK-006（decide.rs） |
| REQ-011（人类评论回喂 inputs） | TASK-002（read_decision_comments）, TASK-004（assemble 回喂） |
| REQ-012（run→decide→run 端到端） | TASK-006 |

未覆盖需求：无。

## Notes

- **类型全为 backend**：本迭代纯 Rust CLI + core 库，无前端 / infra / docs 变更（PRD/engine.md 措辞对齐由归档阶段处理，不占本迭代任务）。
- **已确认边界落到任务**：
  - 裁决①（客观检查器排除）：TASK-005 的 `next_action` 在 Generate 与关卡间**预留检查步位**但本迭代空过——生成后直接置关卡，不写 `check.jsonl`。
  - 裁决②（验收深度 = 一次 `run → decide → run`）：TASK-006 的 QA 用 `MOJIAN_CLAUDE_CMD` 指向假命令端到端跑通 brief 通路，不要求真实创作物、不要求多关卡链路。
  - 裁决③（VOID 最小语义）：TASK-005 的 `apply_decision` VOID 仅写 `void_record` + `chapter.status: void → planned`，不做圣经级联 / 过期检测。
- **可测试性硬约束**：core 单元/集成测用 `FakeRunner`（不 spawn 进程）；CLI E2E 用 `MOJIAN_CLAUDE_CMD` 指向假命令（不触达真实 claude、不花 token）。两条注入路径分别在 TASK-003 与 TASK-006 落测。
- **无 DB 迁移**：`SCHEMA_VERSION` 保持 1；关卡 pending 标记落 `project_state.cursors` JSON（免加列）。
- **占位 SPEC 步归属**：`brief-agent.md` + `brief-agent.manifest.toml` 落 TASK-004（manifest 格式与其解析器 co-evolve），随既有部署 / hash 覆盖机制生效，无需改部署代码；manifest 的 `inputs` 必须能在**新建项目部署后即可解析**（指向部署后必然存在的文件），使 `mojian run` 无需手工种子即可跑通 brief 通路。
- **执行顺序建议**：TASK-002 与 TASK-003 无相互依赖，均在 TASK-001 完成后可并行就绪；builder 单任务串行推进即可。
```