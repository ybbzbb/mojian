# Task File Template

## 命名规范

```text
docs/work/iterations/ITER-xxx/tasks/TASK-{NNN}-{slug}.md
```

---

## 模板

```markdown
# TASK-{NNN} {标题}

- iteration: ITER-xxx
- status: planned
- type: {backend | frontend | infra | docs}
- owner: builder-agent
- created: YYYY-MM-DD
- updated: YYYY-MM-DD

## Goal

一段话说明：解决什么问题，产出是什么。

## Allowed Files

- path/glob/**
- 禁止：path/**

## Inputs

本任务特有的参考信息（type 约定的基础文件无需在此列出）：

- docs/product.md#SectionName — 说明为什么需要这个章节
- docs/work/iterations/ITER-xxx/requirements.md — 说明

## Builder Exit Criteria

builder-agent 自检条目，对应 ralph loop 的出口。每条必须能通过读代码 / 运行类型检查 / 跑单元测试直接验证，不需要启动服务。

- [ ] {具体技术验证项，如：类型检查通过}
- [ ] {单元测试覆盖正常路径}
- [ ] {接口实现满足 tech-design.md 第 N 节定义}

## QA Verification

qa-agent 启动 dev 环境后实际跑的验收条目。**只允许 API / CLI / 日志检查；禁止浏览器自动化与任何 UI 操作**（前端任务的验证以接口契约 + 静态构建为准）。

- [ ] 启动后 GET /health 返回 200
- [ ] curl POST /xxx 带 {payload} 返回 {期望状态码 + 字段}
- [ ] 错误路径：传 invalid input，返回 400 + 指定错误信息

## Dependencies

- 前置任务：TASK-xxx 或 无

## Log

- YYYY-MM-DD [agent] status A → B：说明
```

---

## Type 约定

`type` 决定 agent 在执行前必须额外读取的**基础文件集**，这是 protocol 级别的约定，所有 agent 遵守，不需要在 `Inputs` 中重复声明。

| type | agent 基础读取集 |
| ---- | --------------- |
| `backend` | 项目级技术设计基线（优先 `docs/tech-design/*.md`，否则 `docs/tech-design.md`） + 迭代 `tech-design.md` + `docs/naming.md` + `docs/failure_pattern.md` + `docs/specs/devops.md` |
| `frontend` | 项目级技术设计基线（优先 `docs/tech-design/*.md`，否则 `docs/tech-design.md`） + 迭代 `tech-design.md` + `docs/design.md` + `docs/naming.md` + `docs/failure_pattern.md` + `docs/specs/devops.md` |
| `infra` | 项目级技术设计基线（优先 `docs/tech-design/*.md`，否则 `docs/tech-design.md`） + 迭代 `tech-design.md` + `docs/specs/devops.md` |
| `docs` | `docs/product.md` + `docs/naming.md` |

**`fullstack` 不是合法的 type。**
涉及前后端的需求必须由 planning-agent 拆分为独立的 `backend` 任务和 `frontend` 任务，
frontend 任务设置 `Dependencies` 指向对应的 backend 任务。

---

## 字段规则

| 字段 | 填写方 | 说明 |
| ---- | ------ | ---- |
| `status` | 各 Agent | 初始值 `planned`，流转见 builder_driver.md |
| `type` | planning-agent | 决定 agent 基础读取集，不允许 fullstack |
| `owner` | planning-agent | 执行责任方 |
| `Allowed Files` | planning-agent | glob 格式，builder 不得超出 |
| `Inputs` | planning-agent | 任务特有的补充文件，不写 type 约定已包含的基础文件 |
| `Builder Exit Criteria` | planning-agent | **builder 的 ralph loop 出口**。每条可通过读代码/类型检查/单测验证，不需起服务。禁止写"功能正常"、"显示正确"等主观描述。 |
| `QA Verification` | planning-agent | **qa-agent 启动 dev 环境后真跑的验收条目**。只允许 API / CLI / 日志检查；禁止浏览器自动化、禁止任何 UI 操作步骤。每条须有具体 endpoint、payload、期望返回。 |
| `Log` | builder-agent + qa-agent | 每次状态变更追加 |

---

## 双段 AC 的写法对照

| 错误写法（旧） | 正确写法（新） |
|----------------|----------------|
| `[ ] 接口功能正常` | Builder：`[ ] POST /api/users 返回 201 + 包含 id 字段`<br>QA：`[ ] curl POST /api/users -d '{"name":"x"}' 返回 201` |
| `[ ] 页面显示正确` | Builder：`[ ] 类型检查通过；npm run build 成功`<br>QA：（前端任务通常仅 builder 自检，QA 验后端契约） |
| `[ ] 符合设计规范` | Builder：`[ ] 命名遵循 docs/naming.md`<br>QA：（不验主观规范） |

---

## 边界归属

- 任何**需要启动服务、调接口、跑端到端**的事 → `QA Verification`
- 任何**纯静态、看代码、跑单测**的事 → `Builder Exit Criteria`
- 含糊不清的项 → planning-agent 必须改写到具体可验

builder 不得动 `QA Verification`；qa 不得动 `Builder Exit Criteria`。
