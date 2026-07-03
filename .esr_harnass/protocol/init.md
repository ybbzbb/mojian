# esr_harnass — Project Init

本文件定义 esr_harnass 的目录约定，以及业务项目 `docs/*.md` 的初始化方式。

`install.sh` 只负责安装框架内部文件，不负责替业务项目写 `docs/*.md`。  
如果业务项目文档缺失、为空、或明显过期，应由用户按本文件触发初始化或补全。

---

## 目录约定

```text
{project_root}/
  CLAUDE.md
  README.md
  .esr_harnass/
    protocol/
      builder_driver.md
      init.md
      task-template.md
    guides/
      product.md
      tech-design.md
      design.md
      devops.md
      infra.md
      naming.md
      failure_pattern.md
  .claude/
    agents/
      requirements-agent.md
      design-agent.md
      planning-agent.md
      builder-agent.md
      qa-agent.md
      archivist-agent.md
    skills/esr-api-controller/SKILL.md
    skills/esr-naming/SKILL.md
  docs/
    product.md
    tech-design.md              ← 单文件模式
    tech-design/                ← 大型设计可改用目录模式（二选一）
      *.md
    design.md                     ← 仅 UI 项目需要
    devops.md
    infra.md
    naming.md
    failure_pattern.md
    work/
      current-iteration.md
      history.md
      iterations/
        ITER-xxx/
          meta.md
          gap-analysis.md
          requirements.md
          human-review.md
          ops.md
          tech-design.md
          plan.md
          log.md
          review.md
          archive-proposal.md
          tasks/
            TASK-xxx-slug.md
```

---

## 初始化原则

1. `docs/*.md` 是业务项目当前状态，不是框架模板副本。
2. 这些文件缺失时，应该根据项目真实情况生成；未知项可以写 `TODO`，不能瞎猜。
3. `.esr_harnass/guides/*.md` 是框架内部指导文档，告诉 AI 如何生成和维护业务文档。
4. `docs/work/current-iteration.md` 与 `docs/work/history.md` 也属于项目数据，首次初始化时一并生成。

---

## 何时触发初始化

以下任一情况成立时，要求用户执行初始化：

- `docs/product.md` 不存在或为空
- `docs/tech-design.md` 与 `docs/tech-design/` 同时都不存在
- 使用 `docs/tech-design.md` 时文件为空
- 使用 `docs/tech-design/` 时目录下无任何 `.md` 文件
- `docs/devops.md` 不存在或为空
- `docs/infra.md` 不存在或为空
- `docs/naming.md` 不存在或为空
- `docs/failure_pattern.md` 不存在或为空
- `docs/work/current-iteration.md` 不存在
- `docs/work/history.md` 不存在

`docs/design.md` 仅在 UI 项目中要求存在。

---

## 初始化动作

在项目根目录中，让 Claude 执行以下工作：

1. 创建目录：
   - `docs/`
   - `docs/work/`
   - `docs/work/iterations/`
2. 读取这些输入：
   - `README.md`
   - 现有代码与目录结构
   - `.esr_harnass/guides/product.md`
   - `.esr_harnass/guides/tech-design.md`
   - `.esr_harnass/guides/devops.md`
   - `.esr_harnass/guides/infra.md`
   - `.esr_harnass/guides/naming.md`
   - `.esr_harnass/guides/failure_pattern.md`
   - `.esr_harnass/guides/design.md`（仅 UI 项目）
3. 基于项目真实情况生成或补全：
   - `docs/product.md`
   - `docs/tech-design.md`，或 `docs/tech-design/*.md`
   - `docs/devops.md`
   - `docs/infra.md`（只写静态环境事实；与 `docs/devops.md` 内容不重叠：流程动作、操作步骤只写在 devops.md）
   - `docs/naming.md`
   - `docs/failure_pattern.md`
   - `docs/design.md`（仅 UI 项目）
   - `docs/work/current-iteration.md`
   - `docs/work/history.md`

---

## 推荐提示词

```text
请按 .esr_harnass/protocol/init.md 初始化当前项目文档。

先读取：
- README.md
- 当前仓库代码和目录结构
- .esr_harnass/guides/product.md
- .esr_harnass/guides/tech-design.md
- .esr_harnass/guides/devops.md
- .esr_harnass/guides/infra.md
- .esr_harnass/guides/naming.md
- .esr_harnass/guides/failure_pattern.md
- .esr_harnass/guides/design.md（如果这是 UI 项目）

然后生成或补全：
- docs/product.md
- docs/tech-design.md，或 docs/tech-design/*.md
- docs/devops.md
- docs/infra.md
- docs/naming.md
- docs/failure_pattern.md
- docs/design.md（如果适用）
- docs/work/current-iteration.md
- docs/work/history.md

要求：
- 必须基于项目真实情况写，不能瞎猜
- 缺信息时写 TODO，并明确缺什么
- 输出给人看，同时保证 sub-agent 可直接读取理解
- `docs/infra.md` 只写静态环境事实（服务器拓扑、端口、部署参数等），内容与 `docs/devops.md` 不重叠；流程动作、操作步骤只写在 devops.md
```

---

## `docs/work` 初始内容

`docs/work/current-iteration.md`

```markdown
# Current Iteration

id:
phase: needs_review
source:

## Cursors

product_md_hash:
issue_last_comment_at:
gap_revision: 0
```

`docs/work/history.md`

```markdown
# Iteration History

| Iteration | Opened | Closed | Status | Summary |
|-----------|--------|--------|--------|---------|
```

---

## 文件职责速查

初始化时由 AI 按 `.esr_harnass/guides/*.md` 生成，未知项标 `TODO`，不瞎猜。后续由 agent 在迭代归档时更新，人工可随时修正。

| 文件 | 谁维护 | 用途 |
|------|--------|------|
| `docs/product.md` | init 生成，archivist 归档更新 | 当前产品快照 |
| `docs/tech-design.md` / `docs/tech-design/*.md` | init 生成，archivist 归档更新 | 项目级技术约束 |
| `docs/design.md` | init 生成（仅 UI 项目），人工维护 | 当前 UI 设计规范 |
| `docs/devops.md` | init 生成，人工维护，AI 可协助补全 | 构建验证、部署流程、运维命令、故障排查等流程动作 |
| `docs/infra.md` | init 生成，人工维护，AI 可协助补全 | 静态环境事实：服务器拓扑、端口、部署参数、常驻进程等（与 devops.md 不重叠） |
| `docs/naming.md` | init 生成，人工维护，AI 可协助补全 | 命名与编码规范 |
| `docs/failure_pattern.md` | init 生成，人工维护，AI 可协助整理 | 失败模式知识库 |
| `docs/work/current-iteration.md` | init 生成，agent 维护 | 当前迭代指针 |
| `docs/work/history.md` | init 生成，requirements / archivist 维护 | 迭代历史 |
| `.esr_harnass/guides/*.md` | 框架维护者 | 生成上述业务文档的内部规则 |

---

完整状态机与 phase 约定见 `.esr_harnass/protocol/builder_driver.md`。
