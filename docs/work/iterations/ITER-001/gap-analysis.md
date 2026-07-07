# GAP Analysis — ITER-001

date: 2026-07-07
revision: 1
source: issue#9,#10
status: confirmed

## 需求摘要

搭起 mojian 执行器的最小运行地基：一次性建立 Cargo workspace，落地领域状态机类型与客户端中央 SQLite schema，让 `mojian new` 能建项目、登记项目、部署 SPEC、写 `mojian.toml`，让 `mojian status` 能读中央 DB 输出当前 SOP phase。本迭代只求「运行环境就位」，创作循环相关能力全部留待后续迭代。

## 功能 GAP

| 功能点 | 当前状态 | 目标状态 | 差距 |
|--------|---------|---------|------|
| Cargo workspace | 无任何 Rust 代码（仅 docs/ + LICENSE + 框架目录） | `crates/mojian-core`（库）+ `crates/mojian-cli`（二进制 `mojian`），`cargo build --workspace` 通过 | 全新搭建 |
| 领域状态机类型 | 无 | `SopPhase`（SOP①②③ 两级阶段）、`ChapterState`（七态 + Void） | 全新，枚举变体须逐字对齐 naming.md |
| 客户端中央 DB schema | 无 | `central.db` 建全部 12 表 + `schema_meta` 迁移 | 全新，DDL 已在 storage.md 五节给全 |
| CLI `new`（实现） | 无 | 建项目目录 + 中央 DB 登记 project + 写 `mojian.toml`（含 project_id） | 全新 |
| CLI `status`（实现） | 无 | 读中央 DB，输出该项目当前 SOP phase | 全新 |
| CLI `run` / `decide`（桩） | 无 | 留桩（stub），可被调用但不执行实际逻辑 | 全新（仅占位） |
| 客户端 SPEC 主副本 | 无 `spec/` 目录 | 客户端侧 SPEC 主副本（sop-1/2/3）+ 版本 + hash | 全新 |
| SPEC 部署（`new` 时） | 无 | `mojian new` 把主副本拷进项目 `.claude/agents` / `.claude/skills` / `CLAUDE.md` / `prompts` | 全新 |
| 启动 hash 覆盖 | 无 | 打开项目时比对项目 SPEC 缓存 hash vs 客户端权威，不一致直接覆盖重部署，一致则不写 | 全新（选项 A：项目内 SPEC 纯可弃） |

## 系统 / 架构 GAP

| 层级 | 当前状态 | 需要变更 | 影响范围 |
|------|---------|---------|---------|
| 数据模型 | 无持久化 | 客户端中央 SQLite `central.db`，按 storage.md 五节建 12 表（project / project_state / reference_book / volume / batch / chapter / artifact_ref / bible_version / void_record / stat / config / schema_meta）；`schema_meta.schema_version` 驱动迁移 | 奠基，后续所有迭代依赖 |
| 领域模型 | 无 | `mojian-core` 承载两级状态机类型与实体枚举，供全项目引用 | 奠基 |
| CLI 接口 | 无 | `mojian` 二进制：`new` / `status` 实现，`run` / `decide` 桩 | 用户唯一入口 |
| SPEC 部署 / 运行环境 | 无 | 客户端主副本 → 项目部署缓存；启动 hash 一致性维护；project 表记录 spec_version / spec_hash | 跨「客户端 / 项目」两作用域 |
| 前端模块 | N/A（纯 CLI 工具，无前端） | 无 | — |

## 可行性结论

**结论：** ✅ 可直接实现

- 项目级技术设计基线（overview / storage / engine）已定稿，storage.md「五、DB 表设计」已给出全部表的 SQL DDL，naming.md 已逐字固定枚举变体，实现路径清晰。
- 本迭代刻意只做「结构 + 建项目 + 读状态 + SPEC 部署」，不碰 token 花费面（SDK / 切片 / 检查器），风险集中在工程搭建而非算法。
- 有两处需在设计阶段明确的落点（不阻塞可行性，见「待确认项」）：客户端数据目录位置、SPEC 主副本初始内容来源。

（不在此进行选项对比 — 选型由 design-agent 在下一阶段输出。）

## product.md 变更建议

无。

`docs/product.md` 目前为空；产品边界的补全建议留待 archivist-agent 在归档阶段处理，本需求阶段不改动。

## 明确排除

本次不在范围内的内容（后续 ITER-002/003）：

- SDK 调用（Rust → 无头 `claude` 子进程）
- 上下文切片装配器 + 输入契约 manifest
- 人机决定循环（`run` / `decide` 的实际逻辑）
- 客观检查器（字数 / 对话占比 / 结构，零 token）
- VOID 机制与过期检测
- 日志文件写入（generation / decision / check 三条 JSONL）
- 题材 SPEC 变体的客户端按项目管理
- 节奏统计（stat 表的写入逻辑）——本迭代只建表，不产数据
