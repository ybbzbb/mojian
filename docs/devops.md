# DevOps

## Config

vcs_platform: gh
review_policy: human_required

## Build Verification

> 代码尚未落地，以下为 Rust workspace 就位后的目标校验命令。当前仓库仅有文档，无可编译代码。

快速校验（必跑，编译 + 类型检查，无需完整构建）：

    cargo check --workspace

打包校验（命中以下文件范围时执行）：

    cargo build --workspace --release

影响打包的文件范围：`Cargo.toml`、`crates/*/Cargo.toml`、`app/`（Tauri 配置与前端资源）

格式与静态检查（建议）：

    cargo fmt --check
    cargo clippy --workspace -- -D warnings

> 在 `cargo` 工程存在前，文档类改动无需运行上述命令。

## 部署方式

> planned，待执行器实现后填写。当前无部署产物。

- CLI 分发：GitHub Release 起步，后续 Homebrew tap —— TODO（脚本待定）
- `mojian-core` 发布至 crates.io 供 Rust 生态复用 —— TODO
- 桌面客户端（Tauri 2）：`cargo tauri build` 产出各平台安装包 —— TODO（后期里程碑）

## 常用运维命令

> 本项目为本地单机创作工具，无常驻服务。运行时状态在项目本地的 SQLite + markdown 文件中，无服务端运维。

- 运行 CLI：`mojian <command>`（命令面待定：`next` / `claim` / `done` / …）—— TODO
- 状态库位置与检查命令 —— TODO（SQLite schema 定稿后补）

## 故障排查

> TODO，待执行器实现后补充（状态机卡死排查、上下文装配核对、统计对账等）。

## 分支策略

单一 `main` 分支为主干，远端 `origin`（`git@github.com:ybbzbb/mojian.git`）。

- 当前阶段（理念定型）：设计讨论在 GitHub issue 进行，文档改动直接提交 `main`
- 所有变更遵循 CLAUDE.md 的 Workflow Rules：先有 issue，再按迭代流程执行（错别字/格式/本文件配置项调整除外）
- 功能实现阶段的分支/PR 规范 —— TODO（并行迭代如启用 worktree，命名 `feature/ITER-NNN`）
