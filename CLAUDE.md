# esr_harnass

Protocol entry point: [.esr_harnass/protocol/builder_driver.md](.esr_harnass/protocol/builder_driver.md)

## 启动必读

**任何代码 / 文档改动前，必须先读 `.esr_harnass/protocol/builder_driver.md`，并确认 `docs/work/current-iteration.md` 的当前 phase。** 未确认迭代状态前，不得开始任何修改。

启动时自动加载以下关键上下文（线上配置 + 当前迭代状态 + 命名规范）：

@docs/devops.md
@docs/infra.md
@docs/work/current-iteration.md
@docs/naming.md

> 说明：install.sh 不创建 `docs/*.md`。`@import` 指向尚不存在的文件时 Claude Code 会**静默跳过**，不报错；待 init 流程（见 `.esr_harnass/protocol/init.md`）生成 `docs/*.md` 后，这些 `@import` 自动生效。`docs/product.md`、`docs/tech-design.md`、`builder_driver.md` 因体积或边界考虑不内联加载，继续以链接形式按需引用。

## Workflow Rules

**所有 bug 修复、优化、功能变更必须先创建 issue，再按迭代流程执行。**

- 禁止讨论后直接修改代码/文档（即使改动很小）
- 禁止事后补 issue
- 创建 issue 后，进入新迭代或通过 Source 变更协议并入当前迭代

**例外**（可直接修改，无需 issue）：
- 纯错别字修正
- 注释/格式调整（不影响逻辑）
- docs/devops.md 中的配置项调整

## Directory Rules

### 禁区：`.esr_harnass/`、`.claude/agents/` 和 `.claude/skills/`

`.esr_harnass/` 是 install.sh 生成的**安装产物目录**，由框架版本管理。

**AI 严禁修改 `.esr_harnass/` 下的任何文件。**

同样，`.claude/agents/` 下的 agent 合同文件也是安装产物，**AI 严禁直接修改**。

`.claude/skills/` 下的 skill 文件同样是安装产物，**AI 严禁直接修改**。

如果 task 文件中的 `Allowed Files` 指向了 `.esr_harnass/`、`.claude/agents/` 或 `.claude/skills/` 下的路径，这是**错误的**——AI 应忽略该路径，这些文件由框架管理，不可修改。
