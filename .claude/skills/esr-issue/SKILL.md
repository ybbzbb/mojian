---
name: esr-issue
description: >
  Use when the user asks to create / file / open a NEW GitHub or GitLab issue
  in an esr_harness (ESR platform) repo — enforces a required type
  (需求 / BUG / 优化) mapped to a cross-platform scoped label
  (type::feature | type::bug | type::refactor), a per-type body template,
  an optional version mapped to a milestone, and automatic platform routing
  (github.com -> gh CLI, git.esrcloud.com -> glab CLI). Always previews the
  assembled issue for human confirmation before creating, and echoes the URL
  after. Do NOT use for agent-to-issue comments (those follow the Issue 通信
  协议), nor for editing / closing existing issues.
---

# ESR Issue 创建规范

本 skill 把"结构化创建 issue"的规范固化下来，供主 session 在人借助 AI 提 issue 时按需加载。目的：让类型、结构、版本一次说清楚，从源头提升作为迭代 source 的 issue 质量。

**适用场景：** 创建一个全新的 GitHub / GitLab issue。

**不适用场景（不应触发本 skill）：**

- Agent 向已有 issue 发表 comment（遵循 Issue 通信协议，见 `.esr_harnass/protocol/driver.md`）
- 编辑已有 issue 的标题/正文/标签
- 关闭已有 issue

---

## 一、类型必填

创建 issue 前，**必须**先确定类型，三选一，不设默认值：

| 类型 | 说明 |
|------|------|
| 需求 | 新增能力 / 新功能诉求 |
| BUG | 现有行为与预期不符 |
| 优化 | 现状可用但存在改进空间（性能、体验、代码质量等） |

类型未确定前不得进入后续步骤（标签映射、模板选择均依赖类型）。

---

## 二、类型 → label 映射（跨平台单一 taxonomy）

三种类型分别映射到**同一套字符串 label**，两平台使用完全相同的字符串，不做平台特化映射：

| 类型 | scoped label 字符串 | GitHub 处理 | GitLab 处理 |
|------|--------------------|-------------|-------------|
| 需求 | `type::feature` | 作为普通字符串 label，**不**映射为 GitHub 原生 `enhancement` | 原生 scoped label |
| BUG | `type::bug` | 作为普通字符串 label，**不**映射为 GitHub 原生 `bug` | 原生 scoped label |
| 优化 | `type::refactor` | 作为普通字符串 label | 原生 scoped label |

GitHub 没有原生 scoped label 语法，`::` 按普通字符串字符处理即可，不需要（也不应该）转换为 GitHub 内置的 `enhancement` / `bug` 标签。GitLab 原生支持 scoped label，`type::*` 会被识别为同一 scope 下互斥的标签。

---

## 三、三套正文模板

按类型选用对应模板，字段顺序固定。模板**只写 What**（背景/现象/期望），不引导写实现方案（How）——这一边界与 requirements-agent 的"只写 What"原则一致。

### 需求（type::feature）

```markdown
## 背景

{为什么需要这个能力/为什么现在提出}

## 目标

{这个需求想达成什么效果，不描述实现方式}

## 验收标准

- {可验证的标准 1}
- {可验证的标准 2}
```

### BUG（type::bug）

```markdown
## 复现步骤

1. {step 1}
2. {step 2}

## 期望

{期望的正确行为}

## 实际

{观察到的错误行为}

## 环境·版本

{运行环境 / 版本号 / 分支 commit 等}
```

### 优化（type::refactor）

```markdown
## 现状

{当前行为/实现是什么样}

## 痛点

{现状带来的具体问题}

## 期望改进

{期望达到什么效果，不描述具体实现方案}

## 影响范围

{涉及哪些模块/用户/场景}
```

---

## 四、版本可选 → milestone

版本字段为**可选项**。填写时关联到 milestone，两平台语义对称，均使用 `--milestone`：

- 版本**为空** → 跳过 `--milestone` 参数，不传递空值
- 版本**非空** → 先确认该 milestone 已在目标仓库/项目中存在，再传入 `--milestone "{版本}"`（两平台均要求 milestone 已存在，否则命令失败）

---

## 五、平台路由

根据目标仓库 URL 的域名自动路由：

| 域名 | 平台 | CLI 工具 |
|------|------|----------|
| `github.com` | GitHub | `gh` |
| `git.esrcloud.com` | GitLab | `glab` |

**esrcloud 命令固定前缀（否则 404）：**

```bash
GITLAB_HOST=git.esrcloud.com glab issue create -R ai/esr_harness \
  --title "{标题}" \
  --description "{按类型模板拼装的正文}" \
  --label "type::feature|type::bug|type::refactor（三选一）" \
  [--milestone "{版本}"]
```

**GitHub 命令：**

```bash
gh issue create \
  --title "{标题}" \
  --body "{按类型模板拼装的正文}" \
  --label "type::feature|type::bug|type::refactor（三选一）" \
  [--milestone "{版本}"]
```

---

## 六、创建前预览确认 + 创建后回显

创建 issue 是对外动作，**必须**遵循以下流程：

1. 收集信息：类型（必填）→ 版本（可选）→ 标题 → 按类型模板逐字段填写正文。
2. **创建前展示完整预览**，逐项列出：
   - 目标平台 + 仓库（如 `GitHub / owner/repo` 或 `GitLab / ai/esr_harness`）
   - 类型
   - label（`type::feature` / `type::bug` / `type::refactor`）
   - 标题
   - 正文（按模板拼装后的完整内容）
   - milestone（若版本为空则显示"无"）
3. 等待人工确认后，才执行对应平台的创建命令。人工未确认前不得执行创建命令。
4. 创建成功后，**回显 issue URL**。
5. 创建失败：原样展示 CLI 报错，不静默重试。

---

## 七、兜底策略

| 场景 | 兜底 |
|------|------|
| GitHub label 不存在 | `gh issue create --label` 在 repo 未预建该 label 时会失败（gh 不像 GitLab 自动建 scoped label）。需先幂等预建三个 `type::*` label：`gh label create "type::feature" --force`（`--force` 使已存在时不报错，`type::bug`/`type::refactor` 同理），或在预览步骤提示人工先在 repo 预建 |
| GitLab label 不存在 | GitLab scoped label 首次使用时自动创建，无需预建 |
| milestone 不存在 | 两平台 `--milestone` 均要求 milestone 已存在，创建前需人工确认已存在；不存在则先在 repo/项目中创建 milestone，或本次跳过 `--milestone` |
| 版本字段为空 | 直接跳过 `--milestone` 参数，不传空字符串 |

---

## 边界声明

本 skill 只覆盖"创建一个全新 issue"这一个动作。与 agent 向 issue 发表 comment（Issue 通信协议）、编辑/关闭已有 issue 等场景无关，也不修改任何协议文件或 agent 合同的行为。
