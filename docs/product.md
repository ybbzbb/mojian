# Product

> mojian = SPEC（提示词资产）+ 确定性执行器（Rust）。理念与架构见 `docs/tech-design/`。本文记录**已落地**的用户可见能力（What）。

## 已落地功能

### 运行环境就位（ITER-001，里程碑）

mojian 执行器骨架落地，作者可以建出并检视一个「运行环境就位」的小说项目：

- **`mojian new <dir>`**：建项目目录、在客户端中央 DB 登记项目（分配 `project_id`）、把 SPEC 主副本部署进项目、写出 `mojian.toml`；初始 SOP 阶段为 `style_sampling`。
- **`mojian status [--path <dir>]`**：读回客户端中央 DB，输出该项目当前的 SOP 阶段；打开时按 hash 比对，SPEC 部署漂移会被自动覆盖还原。
- **`mojian run` / `mojian decide`**：本阶段为桩（打印「stub，将在 ITER-002 实现」并以 exit 0 正常退出），创作生成循环尚未接入。

客户端中央 SQLite（`central.db`）承载全部机器状态（12 表 + `schema_meta` 迁移）；项目目录内只有 SPEC 部署缓存与 `mojian.toml`，不含机器状态。SPEC 本迭代为**占位骨架**，仅验证「部署 + hash 覆盖」通路，真实提示词待 SOP #5/#6/#7 定稿后填充。

## 尚未落地

创作生成循环（SDK 调用、切片装配、人机决定循环、客观检查器、VOID / 过期检测、日志文件写入、题材 SPEC 变体、统计产出）——见 `docs/tech-design/engine.md`，规划自 ITER-002 起。
