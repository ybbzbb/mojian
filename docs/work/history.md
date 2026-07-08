# Iteration History

| Iteration | Opened | Closed | Status | Summary |
|-----------|--------|--------|--------|---------|
| ITER-001 | 2026-07-07 | - | issue_open | 运行环境就位（issue#9+#10）：Cargo workspace + 领域状态机 + 中央 DB schema + `new`/`status` + SPEC 部署/hash 覆盖 |
| ITER-002 | 2026-07-07 | - | issue_open | 生成闭环（issue#11+#12）：切片装配 + 输入契约 manifest + 无头 claude SDK 调用 + generation.jsonl + run/decide 决定循环 + decision.jsonl 回喂 |

### ITER-001

- **摘要**：里程碑「运行环境就位」（捆绑 issue#9+#10）。落地 mojian 执行器骨架——Rust Cargo workspace、领域三级状态机枚举、客户端中央 SQLite（12 表 + `schema_meta` 迁移器）、项目登记 + `mojian.toml`、SPEC 主副本占位骨架 1:1 部署 + blake3 tree hash 漂移覆盖，CLI `new`/`status` 实现、`run`/`decide` 桩；workspace 28 测试 + 真实二进制端到端 QA 全通过。归档：product.md 首次成文；tech-design 基线 4 项实现级细化（storage 主副本布局/数据目录解析、engine hash 定义、overview 技术栈基线）；devops.md 新增 `## Build Verification`（人工授权）。
- **改动**：docs/product.md（首次成文）、docs/tech-design/storage.md、docs/tech-design/engine.md、docs/tech-design/overview.md、docs/devops.md；代码 crates/mojian-core + crates/mojian-cli（workspace 奠基）
- **任务**：TASK-001(workspace 骨架 + 依赖基线 + paths/error), TASK-002(领域三枚举 + DB 文本互转), TASK-003(中央 DB 12 表建库 + schema_meta 迁移器), TASK-004(project 登记 + mojian.toml 读写), TASK-005(SPEC 主副本部署 + blake3 tree hash 漂移覆盖), TASK-006(mojian-cli 命令面 new/status + run/decide 桩)

### ITER-002

- **摘要**：里程碑「生成闭环」（捆绑 issue#11+#12）。`run` / `decide` 从桩转真逻辑，打通生成闭环机制通路——切片装配（TOML sidecar 输入契约 manifest + 符号引用 + 段级/整文件双粒度切片）→ 调无头 `claude` 子进程（`GenerationRunner` trait + `MOJIAN_CLAUDE_CMD` 双层可注入 mock）→ 撞人工关卡即停 → `decide` 三判定（CONFIRMED/REVISE/VOID）推进状态机、评论回喂下一次 bundle；`generation.jsonl` / `decision.jsonl` 落客户端；`status` 扩展显卡点。用注入假命令端到端跑通一次 `run → decide → run`（裁决②），占位 SPEC 只验机制通路；无 DB schema 迁移（关卡标记入 `cursors` JSON）。workspace 全量测试通过（core lib 54 / cli 5 + 各集成套件）。归档：product.md 新增「生成闭环」小节 + 修正 ITER-001 桩描述 + 收敛「尚未落地」清单；tech-design overview 技术栈基线加 `serde_json`、engine manifest 措辞对齐 TOML sidecar + 补 `MOJIAN_CLAUDE_CMD` 可注入。客观检查器 / VOID 圣经级联 / 真实 claude schema / 真实 SOP 提示词归 ITER-003（裁决①③）。
- **改动**：docs/product.md、docs/tech-design/overview.md、docs/tech-design/engine.md；代码 crates/mojian-core（新增 sdk / context / log / engine / state 模块 + error 扩 5 变体）、crates/mojian-cli（run/decide 转真逻辑 + status 扩展）；workspace 新增依赖 serde_json
- **任务**：TASK-001(workspace serde_json + error 5 变体), TASK-002(log：generation/decision.jsonl 写入 + 评论回读), TASK-003(sdk：GenerationRunner trait + ClaudeCliRunner + Bundle/SdkResponse), TASK-004(context：TOML sidecar manifest + 符号解析 + 切片 + assemble_bundle), TASK-005(engine next_action/apply_* + state DB 读写), TASK-006(CLI run/decide 真逻辑 + status 显卡点 + 端到端 run→decide→run)
