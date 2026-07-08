# Iteration History

| Iteration | Opened | Closed | Status | Summary |
|-----------|--------|--------|--------|---------|
| ITER-001 | 2026-07-07 | - | issue_open | 运行环境就位（issue#9+#10）：Cargo workspace + 领域状态机 + 中央 DB schema + `new`/`status` + SPEC 部署/hash 覆盖 |
| ITER-002 | 2026-07-07 | - | review_done | 生成闭环（issue#11+#12）：切片装配 + 输入契约 manifest + 无头 claude SDK 调用 + generation.jsonl + run/decide 决定循环 + decision.jsonl 回喂 |

### ITER-001

- **摘要**：里程碑「运行环境就位」（捆绑 issue#9+#10）。落地 mojian 执行器骨架——Rust Cargo workspace、领域三级状态机枚举、客户端中央 SQLite（12 表 + `schema_meta` 迁移器）、项目登记 + `mojian.toml`、SPEC 主副本占位骨架 1:1 部署 + blake3 tree hash 漂移覆盖，CLI `new`/`status` 实现、`run`/`decide` 桩；workspace 28 测试 + 真实二进制端到端 QA 全通过。归档：product.md 首次成文；tech-design 基线 4 项实现级细化（storage 主副本布局/数据目录解析、engine hash 定义、overview 技术栈基线）；devops.md 新增 `## Build Verification`（人工授权）。
- **改动**：docs/product.md（首次成文）、docs/tech-design/storage.md、docs/tech-design/engine.md、docs/tech-design/overview.md、docs/devops.md；代码 crates/mojian-core + crates/mojian-cli（workspace 奠基）
- **任务**：TASK-001(workspace 骨架 + 依赖基线 + paths/error), TASK-002(领域三枚举 + DB 文本互转), TASK-003(中央 DB 12 表建库 + schema_meta 迁移器), TASK-004(project 登记 + mojian.toml 读写), TASK-005(SPEC 主副本部署 + blake3 tree hash 漂移覆盖), TASK-006(mojian-cli 命令面 new/status + run/decide 桩)
