# esr_harnass

esr_harnass 是一套基于文件状态机的多 Agent 协作框架。六个角色（requirements / design / planning / builder / qa / archivist）通过读写文件完成状态交接，无需直接通信。

目录约定与初始化 → [init.md](init.md)
任务文件格式 → [task-template.md](task-template.md)
Agent 合同 → [.claude/agents/](../../.claude/agents/)
平台配置（vcs_platform）→ [docs/devops.md](../../docs/devops.md) `## Config`

---

## 核心原则

**文件是唯一的状态总线。** Agent 之间不直接通信，通过读写文件完成状态交接。

每个 Agent 完成时必须：

1. 写入产出文件
2. 更新 `current-iteration.md` 的 `phase` 字段（仅限有写入权限的 agent）
3. 输出 `AGENT_DONE` 信号

驱动器本身无记忆，每次启动都从 `current-iteration.md` 的 `phase` 读取状态，可安全从任意中断点恢复。

服务启动、停止、账号入口等运行信息统一读取 `docs/devops.md`。凡是需要启动本地服务、执行 dev 命令、确认账号入口的步骤，都以该文件为准，不自行猜测。

---

## 系统约束

**支持多活跃迭代（通过 Worktree 实现）。**

- 每个 worktree 有独立的 `docs/work/current-iteration.md`，指向各自的 ITER-ID
- 同一 worktree 内，`current-iteration.md` 始终只指向一个 ITER-ID
- 新迭代可在任意 worktree 中开启，无需等待其他 worktree 的迭代完成
- 不使用 worktree 时，仍遵循单迭代约束（向后兼容）

---

## current-iteration.md 格式

```text
# Current Iteration

id: ITER-xxx
phase: {当前阶段}
source: https://git.esrcloud.com/ai/esr_harness/-/issues/6 | product-diff | user-description | (空，尚未分析)

## Cursors

product_md_hash: {git hash of product.md when iter started}
issue_last_comment_at: {ISO8601，最后处理的 issue comment 时间戳}
gap_revision: N
```

`source` 和 `## Cursors` 由 requirements-agent 在创建/更新迭代时写入。其他 agent 更新 `phase` 时保留 `source` 和 `Cursors` 原值，不得修改。`source` 可由 driver 在 Source 变更协议中更新（见下方 Source 变更协议）。

**source 字段格式：**
- 新格式（推荐）：完整 URL，如 `https://git.esrcloud.com/ai/esr_harness/-/issues/6`
- 旧格式（兼容）：`issue#N`，默认使用 `vcs_platform` 配置的平台

---

## Phase 状态机

| phase             | 含义                                                   | 写入方                |
|-------------------|--------------------------------------------------------|-----------------------|
| `needs_review`    | 需求尚未分析                                           | 人                    |
| `review_done`     | GAP+需求初稿完成，等人确认                             | requirements-agent    |
| `design_ready`    | 需求已 CONFIRMED 且 agent 终审通过；等 design-agent    | requirements-agent ✱  |
| `design_review`   | 技术设计初稿完成，等人确认                             | design-agent          |
| `planning_ready`  | 技术设计已 CONFIRMED 且 agent 终审通过；等 planning    | design-agent ✱        |
| `building`        | 任务已就绪，builder/qa 循环                            | planning-agent        |
| `archive_ready`   | 所有任务 done，等 archivist-agent                      | qa-agent              |
| `archive_review`  | 归档建议初稿完成，等人确认                             | archivist-agent       |
| `issue_open`      | 流程完成，等待 source issue 人工关闭（仅 issue 来源）  | archivist-agent ✱     |
| `done`            | 迭代最终关闭                                             | archivist-agent ✱ / driver ✱✱ |

✱ 写入条件：见下方各 review 阶段终审检查规则。人也可手动设置作为逃生口。
✱✱ driver 在 `issue_open` 阶段检测到 source issue 已关闭时自动写入。

**三处 `_review` 节点都遵循同一套反馈循环，但支持条件性自动通过：**

| `_review` 阶段     | 触发 agent           | CONFIRMED 通过后目标 phase |
|--------------------|----------------------|----------------------------|
| `review_done`      | requirements-agent   | `design_ready`             |
| `design_review`    | design-agent         | `planning_ready`           |
| `archive_review`   | archivist-agent      | `done`（非 issue 来源或 issue 已关闭）/ `issue_open`（issue 来源且 issue 仍 open）|

### Review Decision（agent 输出到 human-review.md）

agent 在每个 review 阶段**必须**输出结构化 Review Decision，driver 据此判定是否需要人工审核：

```markdown
## Round N — YYYY-MM-DD [{agent-name} review decision]

阶段: review_done | design_review | archive_review
变更类型: feature | bugfix | refactor | docs | config
影响面: small | medium | large
风险信号:
- {风险点 1}
- {风险点 2}

建议: auto_pass | human_review
```

### 风险评估规则（driver 判定）

当 agent 输出 Review Decision 后，driver 按以下规则判定：

**读取配置：** 从 `docs/devops.md` 的 `## Config` 读取 `review_policy`。

**通用规则：**

| 条件 | 判定 |
|------|------|
| `review_policy` = `human_required` | 强制等待人工 |
| 变更类型 = `config` 且影响面 = `large` | 等待人工 |
| 影响面 = `large` | 等待人工 |
| confidence = `low`（风险信号 ≥ 3 条） | 等待人工 |

**阶段规则：**

| 阶段 | 条件 | 判定 |
|------|------|------|
| `review_done` | 变更类型 = `bugfix` 且影响面 = `small` | `auto_pass` |
| `design_review` | 变更类型 = `bugfix` | `auto_pass`（小 bug 跳过设计） |
| `archive_review` | 无风险信号 | `auto_pass` |

**`review_policy` = `cautious` 时：** 中等风险（影响面 = `medium`）也等待人工。

**兜底：** 不匹配任何规则 → 等待人工。

### `_review` 处理流程

1. agent 生成产物 + Review Decision 写入 `human-review.md`
2. driver 读取 Review Decision，按风险评估规则判定
3. **auto_pass**：
   - agent 在产物中写入 `[auto-pass]` 标记和判定依据
   - 在 `ops.md` 记录：`{agent-name} review auto-pass (reason: {匹配的规则}) — done`
   - agent 直接写入下一目标 phase
   - driver 继续推进到下一阶段
4. **等待人工**：
   - agent 停机，输出硬停机格式（见下方）
   - 等待人工在 `human-review.md` 写 `CONFIRMED` 或 `修订`
5. **人工可覆盖**：
   - AI 判定 auto_pass 时，人可在 `human-review.md` 写 `override: human_review` 中断自动推进
   - 人工可强制设置 `review_policy: human_required` 于 `docs/devops.md`

**通用 CONFIRMED 写入条件（适用于等待人工后收到 CONFIRMED 的情况）：**

1. 人在 `human-review.md` 的 `[human feedback]` 节或 issue comment 中写下 `CONFIRMED`
2. 对应 agent 重新调用后，做最终检查（清单见各 agent 合同）
3. 检查通过 → agent 写入下一目标 phase
4. 检查不通过 → agent 生成新 revision，留在当前 `_review` phase，等下一轮 CONFIRMED

**逃生口：** 人可手动直接写下一目标 phase 跳过 agent 终审。

---

## 任务 Status 状态机

| Status        | 含义                         |
|---------------|------------------------------|
| `planned`     | 已规划，前置依赖未满足       |
| `ready`       | 可执行，等待 builder-agent   |
| `in_progress` | 执行中（长任务可选用）       |
| `blocked`     | 执行中遇到阻塞               |
| `reviewing`   | 已完成，等待 qa-agent 验收   |
| `done`        | 验收通过                     |
| `cancelled`   | 已取消                       |

合法流转：

```text
planned     → ready
planned     → cancelled
ready       → in_progress
ready       → cancelled
in_progress → reviewing
in_progress → blocked
blocked     → in_progress
reviewing   → done
reviewing   → in_progress
```

禁止跳过状态。禁止 builder-agent 直接将任务改为 `done`。

**`planned → ready` 推进条件：** 任务 `Dependencies` 中列出的所有前置任务均为 `done`，由 driver 自动执行推进。planning-agent 在创建任务时必须准确声明依赖，否则任务永远不会被推进。

---

## 完整流程（5 阶段 6 角色）

```text
[人] 描述需求
      ↓
═════════════════════ 阶段 1：需求 ═════════════════════
[requirements-agent]  ← 可多次调用
  → 写 gap-analysis.md / requirements.md (What only)
  → phase: review_done
[系统暂停 — 等人工 CONFIRMED]
  ↓ CONFIRMED 终审通过
[requirements-agent] phase: review_done → design_ready
      ↓
═════════════════════ 阶段 2：技术设计 ═════════════════
[design-agent]  ← 可多次调用
  → 写 迭代 tech-design.md (选型 + API + 风险)
  → phase: design_review
[系统暂停 — 等人工 CONFIRMED]
  ↓ CONFIRMED 终审通过
[design-agent] phase: design_review → planning_ready
      ↓
═════════════════════ 阶段 3：拆任务 ═══════════════════
[planning-agent]
  → 写 plan.md + tasks/*.md (含 Builder Exit + QA Verification 两段 AC)
  → phase: planning_ready → building
      ↓
═════════════════════ 阶段 4：实现 + 验收 ══════════════
[driver — Git 初始化]（首次进入 building，见 Git 工作流协议）
  → 创建 feature/{id} 分支 + push
[builder-agent] ←→ [qa-agent]
  扫描 tasks/ 状态，按任务推进：
    builder-agent: ready → in_progress → reviewing
    driver: 任务 reviewing 后自动 commit
    qa-agent:      reviewing → done / in_progress (启 dev 环境真跑 QA Verification)
      ↓ 所有任务 done
[qa-agent] phase: building → archive_ready
      ↓
═════════════════════ 阶段 5：归档 ═════════════════════
[archivist-agent]  ← 可多次调用
  → 写 archive-proposal.md (对 docs/product.md / docs/tech-design.md 的 diff 建议)
  → phase: archive_review
[系统暂停 — 等人工 CONFIRMED]
  ↓ CONFIRMED 终审通过
[archivist-agent]
  → 应用 diff 到 docs/product.md / docs/tech-design.md
  → 更新 history.md 迭代关闭记录
[driver — Git 收口]（见 Git 工作流协议）
  → git commit "feat(iteration): finalize ITER-xxx"
  → 创建 PR/MR（若不存在）+ git push
  → 非 issue 来源或 issue 已关闭：phase → done
  → issue 来源且 issue 仍 open：phase → issue_open

═════════════════════ 阶段 6（条件）：等待 issue 关闭 ═══
[driver] phase: issue_open（仅 issue 来源）
  → 每次启动时检查 source issue 状态
  → issue 已关闭 → phase: done
  → 人工可声明新 issue 并入或切换（见 Source 变更协议）
  → 变更后可回退到合适阶段重新推进

[系统暂停 — 等人开启下一迭代]
```

---

## 启动 / 继续时的状态块

**每次启动（包括用户输入"继续"时），必须先输出状态块，再执行任何操作：**

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[esr_harnass 状态]
迭代：ITER-xxx
Phase：{phase}
来源：{source，如 issue#42 / user-description}
当前步骤：{正在做什么 或 卡在哪里}
等待内容：{具体等待项，如 "issue #42 人工回复" 或 "human-review.md Round N"}
上次活动：{最近一次写入的文件} ({日期})
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

输出后进入决策树。

---

## 启动决策树

每次启动，读 `current-iteration.md` 的 `phase`：

```text
[通用前置检查]（所有 phase 共享，优先于 phase 专属逻辑）
  → 检查是否存在 source 变更声明（见 Source 变更协议）：
      IF 用户输入包含 "switch-source issue#N" 或 "merge issue#N"       → 执行 Source 变更协议
      IF human-review.md 最新 [human feedback] 包含上述关键词            → 执行 Source 变更协议
      IF 任意 issue 最新 comment 包含 "[switch-source] issue#N" 或 "[merge] issue#N" → 执行 Source 变更协议
      （以上三入口按优先级：用户输入 > human-review.md > issue comment）

  → Issue 前置检查（见 CLAUDE.md Workflow Rules）：
      IF 用户描述的是 bug 修复、优化、功能变更（非纯错别字/注释/格式）
         AND 当前没有对应 issue
         → 停机，提示用户先创建 issue，不得直接开始修改
         → 用户创建 issue 后，进入新迭代或通过 Source 变更协议并入当前迭代

phase: needs_review
  → 提示人描述需求，调用 requirements-agent

phase: review_done | design_review | archive_review（通用 _review 逻辑）
  → 检查 human-review.md 最新 [human feedback] 节的"决定"字段：
      IF 决定 == "override: human_review" → 硬停机，等待人工 CONFIRMED
      IF 决定 == 空                  → 检查最新 Review Decision：
                                         IF 符合 auto-pass 规则 → 调用对应 agent 执行 auto_pass
                                         ELSE → 硬停机，展示摘要，等待人工
      IF 决定 == "修订"              → 调用对应 agent 生成新 revision
      IF 决定 == "CONFIRMED"         → 调用对应 agent 进行终审检查
                                         通过 → 写入目标 phase（见 Phase 状态机表）
                                         不通过 → 新 revision，留在当前 phase
      archive_review 特殊：终审通过后，IF source 为 issue 且 issue 仍 open → phase: issue_open；ELSE → phase: done
  ALSO: 同时检查 issue 最新 comment（若 source 为 issue），同步到 human-review.md 后再判断

  对应关系见 Phase 状态机表：review_done → requirements-agent；design_review → design-agent；archive_review → archivist-agent

phase: design_ready
  → 调用 design-agent（首次设计或继续）

phase: planning_ready
  → 调用 planning-agent

phase: building
  → 扫描 tasks/*.md：
      IF reviewing 任务存在       → 调用 qa-agent
      IF ready 任务存在           → 调用 builder-agent（认领后先写 in_progress）
      IF planned 且依赖已 done   → 将其改为 ready，继续循环
      IF blocked 任务存在         → 停机，通知人处理
      IF 所有任务 done            → 调用 qa-agent，由其推进 phase 至 archive_ready

phase: archive_ready
  → 调用 archivist-agent（首次归档或继续）

phase: archive_review
  → 检查 human-review.md 最新 [human feedback] 节的"决定"字段：
      IF 决定 == "override: human_review" → 硬停机，等待人工 CONFIRMED
      IF 决定 == 空                  → 检查最新 Review Decision：
                                         IF 符合 auto-pass 规则 → 调用 archivist-agent 执行 auto-pass
                                         ELSE → 硬停机，展示归档建议摘要，等待人工
      IF 决定 == "修订"              → 调用 archivist-agent 生成新 revision
      IF 决定 == "CONFIRMED"         → 调用 archivist-agent 进行最终检查
                                       通过后：
                                         IF source 为 issue 且 issue 仍 open → phase: issue_open
                                         ELSE → phase: done

phase: issue_open
  → 检查 source issue 状态（见 Source 变更协议 / Issue 关闭协议）：
      IF issue 已关闭               → phase: done（见 Issue 关闭协议）
      IF 人工声明并入/切换新 issue   → 执行 Source 变更协议
      IF 无新操作                    → 展示 issue 状态和变更选项，等待人工操作

phase: done
  → IF 用户输入为"继续"/空输入/未携带具体需求描述：
      自动扫描两类需求来源（不要求用户先开口）：
        A. PRD 漂移：
             git diff {上一迭代 cursor product_md_hash}..HEAD -- docs/product.md
             （cursor 缺失则回退到 git tag prd-v1-init，再缺失则回退到 git log 首次提交）
             IF 有 diff → 候选 source: product-diff（附 diff 摘要）
        B. Issue tracker：
             IF vcs_platform != none：
               拉取 open 状态、且编号未在 history.md 任一行出现过的 issue
               （github: gh issue list --state open --json number,title,createdAt
                gitlab: glab issue list --state opened）
               IF 有 → 候选 source: issue#N（逐条列编号 + 标题 + 创建时间）
  → 展示输出（按下方格式），等待用户确认或重定向：
        - 上一迭代总结一行
        - 候选来源清单（A/B 两类合并；无候选则明示"无新输入"）
        - 提示："回复编号 / 'product-diff' / 直接描述新需求 / 'skip <编号>' 跳过某 issue"
  → 用户回复后：
        IF 选 issue#N      → 写 source: issue#N，调 requirements-agent
        IF 选 product-diff → 写 source: product-diff，调 requirements-agent
        IF 自由描述         → 写 source: user-description，调 requirements-agent
        IF skip / 忽略     → 在 history.md 备注跳过原因，留在 phase: done
```

---

## Agent 角色

| Agent               | 触发条件                                                 | 职责                                                |
|---------------------|----------------------------------------------------------|-----------------------------------------------------|
| requirements-agent  | phase: needs_review / review_done 或新需求描述           | GAP 分析 + 需求文档（What only），停机等 CONFIRMED  |
| design-agent        | phase: design_ready / design_review                      | 迭代级技术设计（选型 + API + 风险），停机等 CONFIRMED |
| planning-agent      | phase: planning_ready                                    | 任务拆分（含 Builder Exit + QA Verification 两段 AC） |
| builder-agent       | phase: building，任务 status: ready                      | 实现单个任务，跑 Build Verification + Builder Exit 自检 |
| qa-agent            | phase: building，任务 status: reviewing                  | 启 dev 环境真跑 QA Verification；全部完成后推 archive_ready |
| archivist-agent     | phase: archive_ready / archive_review                    | 提议 docs/product.md / docs/tech-design.md 更新，停机等 CONFIRMED；CONFIRMED 后应用并关迭代（issue 来源且 issue open → issue_open） |

完整合同（读什么、写什么、禁止项）→ 各自的 `.claude/agents/` 文件。

---

## 通用 Agent 协议

所有 agent 共享以下协议。agent 合同只写**差异**，不重复通用部分。

### 通用启动序列

每个 agent 启动时，前 4 步固定：

1. `.esr_harnass/protocol/builder_driver.md` — 读取平台配置协议、ops.md 协议、Issue 通信协议、Token Log 协议
2. `docs/devops.md` — 获取 `vcs_platform`（未设置时按平台配置协议自动检测写入）
3. 当前迭代 `ops.md`（若存在）— 按 ops.md 协议处理 pending 步骤（有则处理后停止；未解决不得继续）
4. 按状态块格式输出当前状态

之后各 agent 按自身职责读取特定文件。

### Review Decision 格式（三个 review agent 共享）

```markdown
## Round N — YYYY-MM-DD [{agent-name} review decision]

阶段: review_done | design_review | archive_review
变更类型: feature | bugfix | refactor | docs | config
影响面: small | medium | large
风险信号:
- {风险点 1}
- {风险点 2}

建议: auto_pass | human_review
```

变更类型/影响面判断标准见各 agent 合同（因职责不同，具体含义有差异）。

### auto-pass 流程（三个 review agent 共享结构）

当 Review Decision 符合 auto-pass 规则，且 `human-review.md` 中无 `override: human_review` 时：

1. 将产物文件的 `status` 从 `pending_confirmation` 改为 `confirmed`
2. 更新 `current-iteration.md` → 写入目标 phase（见 Phase 状态机表）
3. 在 `human-review.md` 追加 `[auto-pass]` 节，记录判定依据
4. 在 `ops.md` 记录：`{agent-name} review auto-pass (reason: {规则}) — done`
5. 按 Issue 通信协议发送通知 comment（若 source 为 issue）
6. 输出 `AGENT_DONE`

### CONFIRMED 终审流程（三个 review agent 共享结构）

当最新决定为 `CONFIRMED` 时，**不直接转 phase**，先做最终检查（检查项见各 agent 合同）。

**全部通过：**
1. 更新产物文件 status → confirmed
2. 更新 `current-iteration.md` → 目标 phase
3. 写 `human-review.md` 追加终审通过节
4. 按 Issue 通信协议发送 comment（若 source 为 issue）
5. 输出 `AGENT_DONE`

**任一未通过：**
1. 生成 revision N+1，标注未通过项
2. 留在当前 phase
3. 输出 `AGENT_DONE`

### 通用禁止项（所有 agent 共享）

- 修改 `human-review.md` 的 `[human feedback]` 节
- 将 `test`、`test message`、`ok`、`收到` 等噪音短消息同步为人工反馈
- 在存在 `pending` issue comment 时继续同步新 issue comment、生成新 revision、或推进 phase
- 对同一 `revision N` 重复发送 agent comment
- 发送无 agent 前缀的测试 comment 到正式需求 issue

各 agent 合同中的"禁止"节只列**自身特有的禁止项**，不重复上述通用项。

---

## 任务粒度原则

- builder-agent 每次只认领并完成**一个** `ready` 任务
- qa-agent 每次只验收**一个** `reviewing` 任务
- 多任务时驱动器多次循环，不把多任务塞给一个 agent
- 出错时只回滚单个任务，不影响整个迭代

---

## `_review` 硬停机通用格式

`review_done` / `design_review` / `archive_review` 三种 phase 都遵循同一模板：当 human-review.md 最新决定为空时，输出：

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⛔ 系统已暂停：等待人工确认 {阶段名}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

迭代：ITER-xxx
阶段：{review_done | design_review | archive_review}
文件：{对应 revision 文件的相对路径}

{阶段摘要 — agent 在停机时自填}：
  - {要点 1}
  - {要点 2}

待您操作（任选其一）：
  - 修订：在 human-review.md [human feedback] 写意见，决定填"修订" → 重触发 {对应 agent}
  - 确认：在 human-review.md [human feedback] 写"CONFIRMED" → 重触发 {对应 agent} 终审
  - 紧急：直接将 phase 改为 {目标 phase}（逃生口，跳过 agent 终审）

source 为 issue 时，可在 issue 评论中执行同样的操作（agent 会自动同步）。
```

**三场景对应：**

| 当前 phase       | 对应 agent          | 阶段名      | 摘要文件                                | 目标 phase（逃生口）|
|------------------|---------------------|-------------|-----------------------------------------|---------------------|
| `review_done`    | requirements-agent  | GAP 分析    | iterations/ITER-xxx/gap-analysis.md     | `design_ready`      |
| `design_review`  | design-agent        | 技术设计    | iterations/ITER-xxx/tech-design.md      | `planning_ready`    |
| `archive_review` | archivist-agent     | 归档建议    | iterations/ITER-xxx/archive-proposal.md | `done` / `issue_open` |

---

## 平台配置协议

所有 agent 启动时从 `docs/devops.md` 的 `## Config` 读取 `vcs_platform`：

- `gh` — GitHub，使用 `gh` CLI
- `glab` — GitLab，使用 `glab` CLI
- `none` — 跳过所有 issue 相关能力

如果 `vcs_platform` 不存在或值为空，requirements-agent 自动检测并写入：

```bash
git remote -v
which gh && gh auth status 2>/dev/null
which glab && glab auth status 2>/dev/null
```

- remote 含 `github.com` 且 `gh auth` 成功 → `gh`
- remote 含 `gitlab` 或 `git.esrcloud.com` 且 `glab auth` 成功 → `glab`
- 无法确定 → `none`

已存在且非空则不覆盖。

---

## ops.md 协议

**路径：** `docs/work/iterations/{ITER-ID}/ops.md`
**创建时机：** requirements-agent 创建新迭代时同步创建。

**写入规则：** 凡涉及外部操作（issue comment）或需人工确认后才执行的动作，**先写 ops.md 再执行**，执行后更新状态。

**格式：**

```markdown
# Ops Log — ITER-xxx

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

**Pending ops 处理：** 每个 agent 启动时，读 `ops.md`，若有 `pending` 步骤，优先完成，再执行正常逻辑。完成后输出 `AGENT_DONE: {agent-name} | ITER-xxx | pending-ops-resolved`。

**强制规则：**

- 只要 `ops.md` 中存在当前 agent 的 `pending` 外部操作，该 agent **不得继续**同步新的人工反馈、生成新 revision、推进 phase。
- `pending` 步骤处理失败时，必须立即停机并暴露失败原因；禁止跳过后继续主流程。
- 对 issue 来源迭代，`review_done` 阶段的 agent comment 属于**交付物的一部分**；未成功发出前，本轮不得视为完成。

---

## Issue 通信协议

**触发条件：** `vcs_platform` ≠ `none`，且 `current-iteration.md` 的 `source` 为 issue URL 或 `issue#N`。

**source 即路由：** 所有 agent comment 的目标 issue 从 `source` 字段实时解析。Source 变更协议会更新 `source`，后续 comment 自动路由到新 issue，无需各 agent 额外处理。

**URL 解析：** 从 source URL 的域名自动判断平台（见「多源 Issue 路由协议」）。

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

**前缀约定：** 每条 agent comment 必须以 `[{agent-name}]` 开头。这是区分 agent 输出与人工回复的**唯一依据**——同一账号下，不以该前缀开头且时间戳晚于 `issue_last_comment_at` 的 comment 视为人工反馈。

**测试 comment 限制：**

- 禁止向正式需求 issue 发送无前缀的测试 comment，例如 `test`、`test message`、`hello`、`.`。
- 需要验证 comment 通道时，只允许发送带 agent 前缀的测试 comment，例如 `[requirements-agent] test`。
- 联调 comment 默认发往单独测试 issue，不得发往当前活跃迭代对应的正式 issue。
- 若误发到正式 issue，必须在本轮 `ops.md` 记录该污染，并在后续人工反馈同步时显式忽略。

**issue 编号来源：** 从 `source` 字段解析：
- URL 格式：从 URL 提取（如 `https://git.esrcloud.com/ai/esr_harness/-/issues/6` → `6`）
- 旧格式：`issue#42` → `42`；`issue#42,#43` → 逐一回复

**ops.md 记录：** 每次 comment 前在 ops.md 写入该步骤（`pending`），成功后更新为 `done`，失败保持 `pending`。

**原子性要求：**

1. 先落盘本轮文档产物（如 `gap-analysis.md`、`requirements.md`、`human-review.md`）
2. 立即发送对应 issue comment
3. 发送成功后将该 comment 步骤从 `pending` 改为 `done`
4. 只有 1-3 全部完成后，才能输出本轮 `AGENT_DONE`

禁止出现 `pending (will be sent next)` 这类“计划稍后发送”的收尾状态；若发送失败，必须明确记录错误并停机，等待下次启动先补发。

**幂等要求：**

- agent comment 必须带稳定标识，至少包含 `{agent-name}` + `revision N` 或 `final-check`。
- 发送前应先检查 issue 中是否已存在同一稳定标识的 agent comment；若已存在，则本次视为补记 `ops done`，不得重复发送。

**人工反馈过滤：**

- 下列 comment 默认**不得**视为人工反馈：`test`、`test message`、`test from glab`、`.`、`ok`、`收到`。
- 仅当 comment 明确包含需求修订信息、`CONFIRMED`、或对当前 revision 的有效决策时，才可同步进 `human-review.md`。
- 无法判断是否为有效反馈时，保守处理为”忽略且不推进”，并在 `ops.md` 记录 `human-review.md issue comment sync` 的判定说明。

---

## Git 工作流协议

**职责边界：driver 执行全部 Git 操作。** 各 agent 不直接执行 `git commit`、`git push`、`git checkout`、`gh pr create` 等命令。agent 完成工作后由 driver 在状态机转换点自动触发 Git 操作。

**触发条件：** `vcs_platform` ≠ `none`。

### 分支策略

- 分支命名：`feature/{iteration_id}`（如 `feature/ITER-005`）
- 基于 `master` 创建

### 时序

```text
phase: building（首次进入）
  → driver 前置检查：
      git status --porcelain（若有无关脏改动 → 停机，提示人工处理）
      git branch --show-current（若不在 feature/{id} 且非 master → 停机）
  → driver: git checkout -b feature/{iteration_id}（若分支不存在）
  → driver: git push -u origin feature/{iteration_id}

builder 完成任务 → reviewing（driver 推进任务状态后）
  → driver 前置检查：git status --porcelain
  → driver: git add -A
  → driver: git commit -m "feat(iteration): complete TASK-xxx {short-title}"
  → 若 git diff --cached 为空 → 跳过 commit，在 ops.md 记录 skipped

archivist 归档完成（CONFIRMED / auto-pass，phase → done 或 issue_open 前）
  → driver: git add docs/
  → driver: git commit -m "feat(iteration): finalize ITER-xxx"
  → driver 检查 PR/MR 是否已存在（按 vcs_platform 查询）
      若不存在：gh pr create / glab mr create（title: `ITER-xxx: {标题}`，body 含摘要 + `Closes #N`）
      若已存在：跳过创建
  → driver: git push
  → phase → done 或 issue_open
```

### Commit Message 模板

| 类型 | 格式 | 触发时机 |
|------|------|----------|
| 任务级 | `feat(iteration): complete TASK-xxx {short-title}` | 任务 reviewing 后 |
| 收口级 | `feat(iteration): finalize ITER-xxx` | 归档完成后 |

### 异常处理

| 场景 | 检测方式 | 处理 |
|------|----------|------|
| 工作区脏改动 | `git status --porcelain` 有输出 | 停机，提示人工处理，写 ops.md pending |
| 不在 feature 分支 | `git branch --show-current` 不匹配 | 停机，提示人工切换 |
| push 失败 | `git push` 返回非 0 | 写 ops.md pending，下次启动重试 |
| PR/MR 已存在 | 查询列表匹配 title | 仅 push 更新，跳过创建 |
| commit 无变更 | `git diff --cached` 为空 | 跳过 commit，ops.md 记录 skipped |

### 禁止项

- agent 不得执行 `git merge`、`git rebase` 到 `master`
- agent 不得执行 `git push --force`
- PR/MR 合并由人工操作

---

## Worktree 协议

**前提：** 需要并行迭代时使用。单迭代场景无需 worktree。

### 创建 worktree

```bash
# 命名规范：iter-{ITER-ID 小写}
git worktree add ../esr_harness-iter-007 -b feature/ITER-007
```

### 切换 worktree

```bash
cd ../esr_harness-iter-007
```

### 删除 worktree

```bash
# 回到主目录
cd /Users/esr/codes/me/esr_harness
git worktree remove ../esr_harness-iter-007
```

### 查看所有 worktree

```bash
git worktree list
```

### 约束

- 每个 worktree 必须有独立的 `docs/work/current-iteration.md`
- worktree 分支命名：`feature/{ITER-ID}`
- 完成后合并回 master 并删除 worktree
- worktree 内的 git 操作遵循「Git 工作流协议」

---

## 多源 Issue 路由协议

### URL 解析规则

从 source URL 自动判断平台：

| URL 域名 | 平台 | CLI 工具 |
|----------|------|----------|
| `github.com` | GitHub | `gh` |
| `git.esrcloud.com` | GitLab | `glab` |

### 解析逻辑

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

### 向后兼容

旧格式 `source: issue#N` 仍可识别，默认使用 `vcs_platform` 配置的平台。

### 发送命令（按平台选择）

```bash
# GitHub
gh issue comment {N} --body "[{agent-name}] {内容}"

# GitLab
glab issue note {N} --message "[{agent-name}] {内容}"
```

---

## Token Log 协议

**每个 agent 完成时**，将本次 token 用量追加到迭代 `ops.md` 的 `## Token Log` 表格：

```text
| {agent-name} | {context} | {input} | {output} | YYYY-MM-DD |
```

- `context`：本次执行的具体任务（如 `revision 1`、`TASK-001`、`ITER-xxx close`）
- `input` / `output`：token 数；无法获取时填 `N/A`

**qa-agent 关闭迭代时**，读取所有 token 行，按 agent 分组汇总，输出格式：

```text
迭代 Token 用量：
  requirements-agent:  input N / output N
  planning-agent:      input N / output N
  builder-agent:       input N / output N（共 M 次任务）
  qa-agent:            input N / output N（共 M 次验收）
  ─────────────────────────────────────────
  合计:                input N / output N
```

---

## 其他人工干预入口

| 情况                           | 行为                                 |
|--------------------------------|--------------------------------------|
| 任务 `blocked`                 | 停机，列出阻塞原因，等待人解除后重启 |
| PRD 缺失关键信息               | 停机，列出缺失项，等待人补充         |
| builder-agent 发现需要新依赖   | 停机，展示变更建议，等待人确认       |
| qa-agent 同一任务验收失败 2 次 | 停机，通知人直接介入                 |

---

## Source 变更协议（并入 / 切换）

迭代进行中，人工可声明将新 issue 关联到当前迭代。两种模式：

| 模式 | 命令格式 | 可用阶段 | source 变化 | 回退 |
|------|---------|---------|------------|------|
| **并入** | `[merge] issue#N` | 仅 `issue_open` | 追加（`#42` → `#42,#43`） | 必须回退 |
| **切换** | `[switch-source] issue#N` | 任意 phase | 替换（`#42` → `#45`） | 可选回退 |

### 声明入口

人工通过以下任一方式声明：

1. **issue comment**：回复 `[merge] issue#N` 或 `[switch-source] issue#N`
2. **human-review.md**：在 `[human feedback]` 节填写 `merge issue#N` 或 `switch-source issue#N`
3. **对话中直接声明**（切换模式支持）

### 处理流程

1. driver 读取声明，验证目标 issue 存在且为 open 状态
2. 幂等检查：`meta.md` 的 `## Source Changes` 中是否已存在该 issue 编号，若已存在则跳过
3. 更新 `current-iteration.md` 的 `source` 字段：
   - 并入：追加 `,issue#N`（如 `issue#42,#43`）
   - 切换：替换为 `issue#N`；重置 `issue_last_comment_at` 为当前时间
4. 在 `meta.md` 追加记录：
   ```markdown
   ## Source Changes

   - issue#N
     - mode: merge | switch
     - changed_at: YYYY-MM-DD
     - previous_source: issue#42
     - changed_by: human
     - reason: {原因}
     - rollback_to: building | design_review | review_done | (空，仅切换模式可选不回退)
   ```
5. 人工指定回退层级（agent 不得自动推断）：

   | scope | 回退目标 phase | 说明 |
   |-------|---------------|------|
   | 仅需补任务 | `building` | 原有需求和设计不变，只需增加/修改任务 |
   | 涉及设计差异 | `design_review` | 需求不变但技术方案需调整 |
   | 涉及需求变化 | `review_done` | 需求本身需要修订 |
   | 不回退（仅切换） | 保持当前 phase | 仅切换 source，流程继续 |

6. driver 更新 `current-iteration.md` 的 `phase`（若需回退）
7. 在 `ops.md` 记录：
   ```
   - [x] source-change: issue#42 → issue#45 (mode: switch, phase: building → review_done) — done
   ```

### 边界规则

- 超范围的新需求不得并入当前迭代，必须开新 issue / 新迭代
- 变更判断必须由人工做出，agent 不得自动推断
- 已变更的 issue 不得重复操作

### 回退后行为

- 迭代 ID 不变
- 之前已完成的任务状态保留（除非人工明确要求重做）
- 已归档的 diff（archive-proposal.md）在回退后标记为 `stale`
- 后续所有 agent comment 自动路由到更新后的 source（因 source 已变更）
- 切换模式下 `issue_last_comment_at` 重置，避免旧 issue comment 误判
- 切换模式下旧 issue 的后续 comment 不再被自动同步

---

## Issue 关闭协议（`issue_open` → `done`）

**触发条件：** `phase` 为 `issue_open`，driver 检测到 source issue 已被人工关闭。

**检查方式：**

```bash
# GitHub
gh issue view {N} --json state -q .state

# GitLab
glab issue view {N} --json state | jq -r '.state'
```

**处理流程：**

1. driver 确认 issue 状态为 `closed`
2. 更新 `current-iteration.md` → `phase: done`
3. 更新 `docs/work/history.md`：
   - 本迭代行的 `Closed` 填今天，`Status` 改为 `done`
   - 在表格下方为本迭代追加总结（格式见 archivist-agent.md `history.md 更新规范`）：摘要、改动、任务
4. 在 `ops.md` 记录：`driver issue_open → done (source issue closed) — done`
5. 输出状态块，提示迭代已关闭

**幂等要求：**

- 检查 issue 状态前，先检查 `current-iteration.md` 的 `phase`
- 若 `phase` 已为 `done`，跳过关闭流程
- 若 issue 已关闭但 `phase` 仍为 `issue_open`，执行关闭流程

**与 Issue 通信协议的关系：**

- `issue_open` 阶段的 issue 状态检查**不需要**写 ops.md（非外部通信，只是读取）
- 并入操作的 comment 需要遵循 Issue 通信协议（写 ops.md、带前缀等）
