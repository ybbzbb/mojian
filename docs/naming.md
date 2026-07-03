# Naming

> 代码尚未落地，以下多数为**约定目标**（planned）。已由仓库/设计固化的部分为事实，其余待首个实现迭代确认。

## File / Module Naming

- Cargo crate 目录：`crates/mojian-core`、`crates/mojian-cli`（kebab-case，`mojian-` 前缀）
- Rust 模块 / 文件：`snake_case.rs`
- 提示词包目录：`prompts/sop-N-<slug>/`（如 `sop-1-style`、`sop-2-bible`、`sop-3-writing`）
- 创作产物 markdown：`kebab-case.md`（如 `creative-brief.md`、`creative-vision.md`）
- 迭代/任务命名遵循框架规范：迭代 `ITER-NNN`，任务 `TASK-NNN-slug`

## Function / Variable Naming

- 函数 / 方法：`snake_case`，动词开头（如 `assemble_context`、`next_task`）
- 类型 / 枚举 / trait：`PascalCase`（如 `ChapterState`、`SopPhase`）
- 常量 / 静态：`UPPER_SNAKE_CASE`（如 `MAX_SKELETON_WORDS`）
- 枚举变体命名对齐设计文档的状态名（如章节状态 `Planned` / `SkeletonDrafting` / `SkeletonReview` / `ProseDrafting` / `ProseReview` / `Approved` / `Void`）

## API / CLI Naming

- CLI 子命令：动词 / 短语，kebab-case（拟 `next` / `claim` / `done` / …，命令面 TODO）
- SQLite 表 / 列：`snake_case`（schema 定稿后确定）
- 状态机 phase 名与三个 SOP 设计文档（#5/#6/#7）逐字对齐，代码与文档不得出现两套叫法

## Commit Message

格式：`{type}: {description}`（动词原形开头）

- `feat`: 新功能
- `fix`: bug 修复
- `refactor`: 重构（无功能变更）
- `docs`: 文档变更（当前阶段主要类型）
- `chore`: 工程/构建杂项

## 注释规则

- 不写解释「做了什么」的注释；只在逻辑非常规时写注释说明「为什么」。
- 提示词资产（`prompts/`）以明文自解释为准，避免在 Rust 源码中重复提示词内容。
- 设计决策的理由写进 issue 或 `docs/`，不散落在代码注释里。
