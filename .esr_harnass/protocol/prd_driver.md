# PRD 准入 SOP（SOP-A）协议入口

本文件定义 **PRD 准入 SOP（SOP-A）** 的完整状态机、三个 agent 协作时序、产物清单与通用操作协议。

**与迭代 SOP（SOP-B）完全隔离：**

- PRD SOP 不读写 `current-iteration.md`，不共享迭代 phase 集，不共享编号空间（`PRD-NNN` 与 `ITER-NNN` 独立自增）
- 衔接仅由人工把 `handoff.md` 路径手动作为 source 输入 SOP-B 的新迭代——agent 严禁自动衔接
- 两条 SOP 各自维护独立的 `current-prd.md` / `current-iteration.md`、`human-review.md`、`ops.md`、Token Log

---

## PRD 状态机

| phase | 含义 | 写入方 |
|-------|------|--------|
| `prd_intake` | PRD 已放入 `source.md`，等 prd-summarizer-agent | 人（开启 PRD 时） |
| `summarizing` | prd-summarizer-agent 工作中（可中断 / 可恢复） | prd-summarizer-agent |
| `summarize_review` | `summary.md` 初稿完成，等人确认 | prd-summarizer-agent |
| `mapping` | summary 已 CONFIRMED，等 prd-mapper-agent | prd-summarizer-agent ✱ |
| `mapping_review` | `modules.md` 初稿完成，等人确认 | prd-mapper-agent |
| `gate_review` | `gate-report.md` 初稿完成，等人确认（含 Override Decision 占位节） | prd-gatekeeper-agent |
| `passed` | gate 通过，已生成 `handoff.md`（终态） | prd-gatekeeper-agent ✱ |
| `blocked` | gate 失败（终态，人工可修改上游文件后重跑） | prd-gatekeeper-agent |
| `override` | 人工在 `gate-report.md` 显式写入 override 决定后进入（终态，已生成 `handoff.md`） | 人触发 + prd-gatekeeper-agent 读取后生成 handoff |

✱ 写入条件：对应 `_review` 阶段收到人工 CONFIRMED 后终审通过。人也可手动设置作为逃生口。

合法流转：

```text
prd_intake      → summarizing
summarizing     → summarize_review
summarize_review → mapping（CONFIRMED 终审通过）
mapping         → mapping_review
mapping_review  → gate_review（CONFIRMED 终审通过）
gate_review     → passed（gate 判定通过 + CONFIRMED）
gate_review     → blocked（gate 判定失败 + CONFIRMED）
gate_review     → override（人工在 Override Decision 节写入 decision: override 后重跑）
blocked         → summarize_review | mapping_review | gate_review（人工修改上游后重跑，视修改深度而定）
```

终态：`passed` / `blocked` / `override`（不设"取消/废弃" phase，人工直接删除目录处理）

---

## PRD `_review` Auto-Pass Override

**无论 `docs/devops.md` 的 `review_policy` 取值如何（包括 `auto`）**，PRD SOP 的三个 `_review` phase 一律等待人工 CONFIRMED，driver 不进入 auto-pass 分支：

| PRD `_review` phase | 规则 |
|---------------------|------|
| `summarize_review` | 强制等待人工 CONFIRMED，禁止 auto-pass |
| `mapping_review` | 强制等待人工 CONFIRMED，禁止 auto-pass |
| `gate_review` | 强制等待人工 CONFIRMED，禁止 auto-pass |

三个 PRD agent 仍按通用 Agent 协议在 `human-review.md` 输出 Review Decision（见下方 Review Decision 结构节），供人工参考，但 driver 看到 PRD SOP 的 `_review` phase 时直接进入"等待人工"分支，不做 auto-pass 规则匹配。

---

## Override 入口契约

`gate-report.md` 末尾 `## Override Decision` 节是**唯一**的人工 override 入口。

prd-gatekeeper-agent 首次写 `gate-report.md`（进入 `gate_review` 时）强制在末尾输出以下占位符：

```markdown
## Override Decision

> 本节仅供人工填写。Agent 不得修改本节中由人工填入的字段。
> 如需 override 当前 gate 结果（blocked → override），请填写：
> - decision: override
> - reason: {覆盖理由，必填}
> - by: {人名，必填}
> - at: YYYY-MM-DD

decision:
reason:
by:
at:
```

**agent 重跑时只读不写该节。** 若发现 `decision:` 字段值为 `override` 且 `reason` / `by` / `at` 均非空，则视为人工 override 生效：phase → `override`，并生成 `handoff.md`。

字段约定：

| 字段 | 说明 |
|------|------|
| `decision` | 取值 `override` 时生效；其他值（含空）视为未填写 |
| `reason` | 覆盖理由，必填，不得为空 |
| `by` | 填写人名，必填 |
| `at` | 填写日期，格式 YYYY-MM-DD，必填 |

---

## handoff.md 文件接口

`handoff.md` 在 gate `passed` 或 `override` 后由 prd-gatekeeper-agent 生成，作为人工开启 SOP-B 新迭代时的输入材料。

文件路径：`docs/prd/PRD-NNN/handoff.md`

格式：

```markdown
# Handoff — PRD-NNN

date: YYYY-MM-DD
prd_version: V1.0.0
source_prd: docs/prd/PRD-NNN/source.md
gate_result: passed | override
override_reason: (若 gate_result = override 时填写，对应 Override Decision 节 reason 字段)

## 摘要

（引用 summary.md 的目标、非目标、验收标准、约束的结构化压缩）

## 模块清单

（引用 modules.md 的模块列表、模块边界、模块依赖）

## 关键边界与约束

（对实施团队最重要的禁止项与技术限制，从 summary.md + modules.md 提炼）

## checklist 结果

（引用 gate-report.md 中各 checklist 项的判定结果：通过 / 未通过 / 豁免）
```

**衔接方式（SOP-A → SOP-B）：** 人工在启动 SOP-B 新迭代时，将 `handoff.md` 路径或其内容作为 user-description 类 source 输入给 `requirements-agent`。agent 不得自动衔接，衔接动作必须由人工触发。

---

## 目录结构

PRD 准入 SOP 在业务项目侧使用以下目录结构：

```
docs/prd/
  checklist.md             ← 由 AI 按 .esr_harnass/guides/prd-checklist.md 在首次启动 PRD 流程时生成
  current-prd.md           ← 当前 PRD 指针（PRD SOP 状态指针，与 current-iteration.md 同构但完全隔离）
  history.md               ← PRD 历史表
  PRD-NNN/
    source.md              ← 人工放入的原始 PRD 文档
    summary.md             ← prd-summarizer-agent 产出
    modules.md             ← prd-mapper-agent 产出
    gate-report.md         ← prd-gatekeeper-agent 产出（末尾含 Override Decision 占位节）
    handoff.md             ← prd-gatekeeper-agent 在 passed / override 后产出
    human-review.md        ← agent 写系统节，人写 [human feedback] 节
    ops.md                 ← 三个 PRD agent 共享，含 Token Log
    meta.md                ← 首次 phase 进入时创建，记录元数据
```

`current-prd.md` 格式（与 `current-iteration.md` 同构，但物理隔离、编号隔离）：

```markdown
# Current PRD

id: PRD-NNN
phase: prd_intake | summarizing | summarize_review | mapping | mapping_review | gate_review | passed | blocked | override
source_prd_path: docs/prd/PRD-NNN/source.md

## Cursors

prd_revision: N
issue_last_comment_at: (若 PRD 流程绑定了 issue，否则为空)
```

`history.md` 格式（PRD 历史表）：

```markdown
# PRD History

| PRD | Opened | Closed | Result | Summary |
|-----|--------|--------|--------|---------|
| PRD-001 | YYYY-MM-DD | YYYY-MM-DD | passed | ... |
```

### 首次启动 PRD 流程

业务项目首次启动 PRD 流程时，按以下步骤初始化：

1. 创建 `docs/prd/` 目录
2. 由 AI 按 `.esr_harnass/guides/prd-checklist.md` 生成 `docs/prd/checklist.md`
3. 创建 `docs/prd/current-prd.md`（phase 设为 `prd_intake`）
4. 创建 `docs/prd/history.md`
5. 创建 `docs/prd/PRD-001/` 目录与 `source.md`（人工放入原始 PRD 内容）

---

## 编号规则

**`PRD-NNN` 与 `ITER-NNN` 编号空间完全独立：**

- `PRD-NNN`：三位数字，自增，从 `PRD-001` 开始，由人工在创建新 PRD 目录时分配
- `ITER-NNN`：三位数字，自增，由 requirements-agent 在创建新迭代时分配
- 两套编号互不嵌套、互不引用对方的编号
- `docs/prd/history.md` 与 `docs/work/history.md` 分别记录各自的历史，互不合并

**演进规则：**

- **禁止一个 `PRD-NNN` 衍生出多个迭代**。PRD 内容的演进通过 PRD 文档自身的版本号（如 `V1.0.1`）在 `summary.md` / `source.md` 内管理
- 每次 PRD 实质变更须走一次新的 `PRD-NNN` 流程，而不是复用旧 `PRD-NNN` 开多个迭代
- PRD-NNN 的"取消 / 废弃"不设显式 phase，由人工直接删除目录处理

---

## 三个 PRD Agent 说明

| Agent | 触发条件 | 职责 |
|-------|----------|------|
| `prd-summarizer-agent` | phase: `prd_intake` / `summarize_review` | 读 `source.md`，产出 `summary.md`（目标、非目标、验收标准、约束的结构化压缩）；phase 流转 `prd_intake → summarizing → summarize_review`；CONFIRMED 终审通过后 phase → `mapping` |
| `prd-mapper-agent` | phase: `mapping` / `mapping_review` | 读 `source.md` + `summary.md`，产出 `modules.md`（模块清单、模块边界、模块依赖）；CONFIRMED 终审通过后 phase → `gate_review`（触发 prd-gatekeeper） |
| `prd-gatekeeper-agent` | phase: `gate_review` | 读 `checklist.md` + 上游产物，产出 `gate-report.md`（含 Override Decision 占位节）；判定 `passed` / `blocked`；人工 override 后读取并生成 `handoff.md` 终态 |

**agent comment 前缀约定：**

- `[prd-summarizer-agent]` — prd-summarizer-agent 发出的 issue comment
- `[prd-mapper-agent]` — prd-mapper-agent 发出的 issue comment
- `[prd-gatekeeper-agent]` — prd-gatekeeper-agent 发出的 issue comment

这是区分 agent 输出与人工回复的唯一依据（同一账号下，不以该前缀开头且时间戳晚于 `issue_last_comment_at` 的 comment 视为人工反馈）。

---

## 启动决策树（PRD 版）

每次启动 PRD SOP，读 `docs/prd/current-prd.md` 的 `phase`：

```text
[通用前置检查]
  → 读 ops.md，若有 pending 步骤，优先处理，处理后停机
  → 输出当前状态块

phase: prd_intake
  → 调用 prd-summarizer-agent

phase: summarizing
  → 调用 prd-summarizer-agent（继续 / 恢复）

phase: summarize_review
  → 检查 human-review.md 最新 [human feedback] 节的"决定"字段：
      IF 决定 == 空       → 硬停机，展示摘要，等待人工（PRD SOP 不走 auto-pass）
      IF 决定 == "修订"   → 调用 prd-summarizer-agent 生成新 revision
      IF 决定 == "CONFIRMED" → 调用 prd-summarizer-agent 进行终审检查
                               通过 → phase: mapping
                               不通过 → 新 revision，留在 summarize_review

phase: mapping
  → 调用 prd-mapper-agent

phase: mapping_review
  → 检查 human-review.md 最新 [human feedback] 节的"决定"字段：
      IF 决定 == 空       → 硬停机，等待人工（PRD SOP 不走 auto-pass）
      IF 决定 == "修订"   → 调用 prd-mapper-agent 生成新 revision
      IF 决定 == "CONFIRMED" → 调用 prd-mapper-agent 进行终审检查
                               通过 → phase: gate_review
                               不通过 → 新 revision，留在 mapping_review

phase: gate_review
  → 检查 gate-report.md 末尾 Override Decision 节：
      IF decision == "override" 且 reason / by / at 均非空
        → 调用 prd-gatekeeper-agent 读取 override，生成 handoff.md，phase: override
      ELSE 检查 human-review.md 最新 [human feedback] 节：
        IF 决定 == 空       → 硬停机，等待人工（PRD SOP 不走 auto-pass）
        IF 决定 == "修订"   → 调用 prd-gatekeeper-agent 生成新 revision
        IF 决定 == "CONFIRMED" → 调用 prd-gatekeeper-agent 进行终审检查
                                 通过（passed）→ 生成 handoff.md，phase: passed
                                 通过（blocked）→ phase: blocked
                                 不通过 → 新 revision，留在 gate_review

phase: passed | blocked | override（终态）
  → 展示终态摘要，提示人工：
      passed / override：handoff.md 已就绪，人工可携带路径开启 SOP-B 新迭代
      blocked：上游文件修改后可重跑 prd-gatekeeper-agent 或相应上游 agent
  → 停机，等待人工操作
```

---

## 通用片段（内联副本）

以下内容复制自 `protocol/builder_driver.md` 当前版本，作为 PRD SOP 的通用操作基础。两条 SOP 各维护自己的副本；未来修改通用片段时需同步更新。

### 平台配置协议

所有 PRD agent 启动时从 `docs/devops.md` 的 `## Config` 读取 `vcs_platform`：

- `gh` — GitHub，使用 `gh` CLI
- `glab` — GitLab，使用 `glab` CLI
- `gh+glab` — 同时支持 GitHub 和 GitLab，从 source URL 自动路由
- `none` — 跳过所有 issue 相关能力

**URL 解析规则：**

| URL 域名 | 平台 | CLI 工具 |
|----------|------|----------|
| `github.com` | GitHub | `gh` |
| `git.esrcloud.com` | GitLab | `glab` |

```bash
# 从 URL 提取平台和 issue 编号
if [[ "$SOURCE_URL" == *"github.com"* ]]; then
  PLATFORM="gh"
  ISSUE_NUM=$(echo "$SOURCE_URL" | grep -oE '/issues/[0-9]+' | grep -oE '[0-9]+')
elif [[ "$SOURCE_URL" == *"git.esrcloud.com"* ]]; then
  PLATFORM="glab"
  ISSUE_NUM=$(echo "$SOURCE_URL" | grep -oE '/issues/[0-9]+' | grep -oE '[0-9]+')
fi
```

旧格式 `source: issue#N` 仍可识别，默认使用 `vcs_platform` 配置的第一个平台。

### ops.md 协议

**路径：** `docs/prd/PRD-NNN/ops.md`（PRD SOP 版本，与 SOP-B 的 `docs/work/iterations/ITER-NNN/ops.md` 同结构）
**创建时机：** prd-summarizer-agent 创建新 PRD 目录时同步创建。

**写入规则：** 凡涉及外部操作（issue comment）或需人工确认后才执行的动作，先写 ops.md 再执行，执行后更新状态。

**格式：**

```markdown
# Ops Log — PRD-NNN

## {agent-name} — YYYY-MM-DD

steps:
- [x] {步骤描述} — done
- [ ] {步骤描述} — pending (attempted YYYY-MM-DD, error: {原因})
- [ ] {步骤描述} — awaiting_approval

## Token Log

| agent | context | input | output | date |
|-------|---------|-------|--------|------|
```

**步骤状态值：**

- `done` — 已完成
- `pending` — 失败，下次启动第一步优先重试
- `awaiting_approval` — 等待人工确认后执行
- `skipped` — 不适用（如 vcs_platform: none 时跳过 comment）

**Pending ops 处理：** 每个 PRD agent 启动时，读 `ops.md`，若有 `pending` 步骤，优先完成，再执行正常逻辑。

**强制规则：**

- 只要 `ops.md` 中存在当前 agent 的 `pending` 外部操作，该 agent 不得继续推进 phase
- `pending` 步骤处理失败时，必须立即停机并暴露失败原因

### Issue 通信协议

**触发条件：** `vcs_platform` ≠ `none`，且 PRD 流程绑定了 issue（`current-prd.md` 的 `issue_last_comment_at` 非空）。

**发送命令：**

```bash
# GitHub
gh issue comment {N} --body "$(cat <<'COMMENT'
[{agent-name}] {正文内容}
COMMENT
)"

# GitLab
glab issue note create {N} --message "[{agent-name}] {正文内容}"
```

**前缀约定：** 每条 PRD agent comment 必须以 `[prd-summarizer-agent]` / `[prd-mapper-agent]` / `[prd-gatekeeper-agent]` 开头。

**ops.md 记录：** 每次 comment 前在 ops.md 写入该步骤（`pending`），成功后更新为 `done`，失败保持 `pending`。

**幂等要求：** agent comment 必须带稳定标识（agent-name + revision N 或 final-check）；发送前检查是否已存在同一标识的 comment，若存在则补记 ops done，不重复发送。

**人工反馈过滤：** 下列 comment 不得视为人工反馈：`test`、`test message`、`.`、`ok`、`收到`。仅当 comment 明确包含修订信息、`CONFIRMED` 或有效决策时，才可同步进 `human-review.md`。

### Token Log 协议

**每个 PRD agent 完成时**，将本次 token 用量追加到 `docs/prd/PRD-NNN/ops.md` 的 `## Token Log` 表格：

```text
| {agent-name} | {context} | {input} | {output} | YYYY-MM-DD |
```

- `context`：本次执行的具体任务（如 `summarize revision 1`、`PRD-001 gate-check`）
- `input` / `output`：token 数；无法获取时填 `N/A`

PRD SOP 的 Token Log 统计在 `docs/prd/PRD-NNN/ops.md`，与 SOP-B 的 `docs/work/iterations/ITER-NNN/ops.md` 分别记录，互不合并。

### Review Decision 结构

三个 PRD agent 在每个 `_review` 阶段必须输出结构化 Review Decision 到 `human-review.md`（供人工参考，PRD SOP 不走 auto-pass）：

```markdown
## Round N — YYYY-MM-DD [{prd-agent-name} review decision]

阶段: summarize_review | mapping_review | gate_review
变更类型: feature | bugfix | refactor | docs | config
影响面: small | medium | large
风险信号:
- {风险点 1}
- {风险点 2}

建议: human_review
```

注意：PRD SOP 的 `建议` 字段**始终为 `human_review`**，即使按 builder_driver.md 的通用风险评估规则本可 auto-pass，PRD SOP 也禁止 auto-pass（REQ-010 / CON-007）。

---

## 禁止项

以下禁止项约束三个 PRD agent 的行为，任何 agent 均不得违反：

- **agent 不得自动衔接 PRD SOP 与迭代 SOP**：不写 `current-iteration.md`、不调用 `requirements-agent`、不创建新迭代目录、不将 `handoff.md` 路径自动输入 SOP-B
- **agent 不得修改 `gate-report.md` 末尾 `## Override Decision` 节中由人工填入的字段**：`decision` / `reason` / `by` / `at` 四个字段一旦由人工填写，agent 只读不写
- **agent 不得自动选择或建议 `override`**：override 决定完全由人工在 Override Decision 节填写，agent 只在检测到合法 override 填写后执行后续生成动作
- **agent 不得自动清理或建议清理 `docs/prd/PRD-NNN/` 目录**：PRD-NNN 的废弃由人工直接删除目录处理，agent 不得触发或建议任何清理动作
- **agent 不得在 PRD `_review` phase 触发 auto-pass**：无论 `review_policy` 取值（含 `auto`），三个 `_review` phase 一律等待人工 CONFIRMED
- **agent 不得实现自动级联失效或自动重跑上下游**：人工修改任意上游文件后，下游产物视为陈旧，是否重跑由人工判断并手动触发
- **agent 不得对任意 PRD `_review` phase 进行 auto-pass 规则匹配**：driver 看到 PRD SOP 的 `_review` phase 时直接等待人工，不调用 auto-pass 分支
- **agent 不得将一个 PRD-NNN 衍生出多个迭代**：每次 PRD 实质变更须开新 PRD-NNN，不复用旧编号
