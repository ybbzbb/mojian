# Archive Proposal — ITER-002

date: 2026-07-08
revision: 1
status: confirmed

## 迭代成果概览

完成任务：6 个
取消任务：0 个
关键产出：
- TASK-001：workspace 新增唯一依赖 `serde_json`（`[workspace.dependencies]`）+ `mojian-core::error` 扩 5 变体（子进程失败 / JSON 解析 / manifest 非法 / 符号无法解析 / 关卡状态不匹配）。
- TASK-002：`mojian-core::log` — `generation.jsonl` / `decision.jsonl` 逐行追加写 + `read_decision_comments` 评论回读（REQ-011）。
- TASK-003：`mojian-core::sdk` — `GenerationRunner` trait + `ClaudeCliRunner`（`std::process::Command`，基础命令读 `MOJIAN_CLAUDE_CMD`）+ `Bundle` 五字段 + `SdkResponse`（result/cost/usage 容错解析）。
- TASK-004：`mojian-core::context` — TOML sidecar manifest 读取 + 符号 `<source>.<selector>[:{params}][#anchor]` 手写解析 + 段级/整文件双粒度切片 + `assemble_bundle`；占位 `brief-agent.md` + `brief-agent.manifest.toml`。
- TASK-005：`mojian-core::engine`（纯函数 `next_action` + `apply_generation` / `apply_decision` + `Verdict`）+ `state`（运行时 DB 行读写：sop_phase 推进 / 关卡入 `cursors` JSON / chapter 状态 / void_record / artifact_ref）。
- TASK-006：CLI `run` / `decide` 从桩转真逻辑、`status` 扩展显卡点；`MOJIAN_CLAUDE_CMD` 注入假命令端到端跑通一次 `run → decide → run`（裁决②）。

端到端结论：workspace 全量测试通过（core lib 54 / cli 5 / 各集成套件），无 DB schema 迁移。

## 对 docs/product.md 的修改建议

### 章节「已落地功能」：新增 ITER-002 里程碑小节

**原文（在 ITER-001 小节之后、「尚未落地」之前插入）：**

```
（ITER-001 小节结束）
```

**新文（新增一节）：**

```
### 生成闭环（ITER-002，里程碑）

`run` / `decide` 从桩转为真逻辑，打通「生成闭环」的机制通路——装配切片 → 调（可替换的）生成命令 → 撞人工关卡即停 → 人给决定后推进 / 回喂：

- **`mojian run [--path <dir>]`**：定位项目 → 打开时按 hash 覆盖还原 SPEC → 循环状态机 `next_action`；到 Generate 步就按输入契约 manifest 装配最小切片 bundle（SPEC 切片 + SSOT 切片 + 前情），调无头 `claude` 子进程生成、把结果与 token/成本追加进 `generation.jsonl`，然后停在人工关卡；无可跑动作时正常退出。
- **`mojian decide <关卡> <判定> [目标] [--comment "..." | --file <path>]`**：三判定 `CONFIRMED` / `REVISE` / `VOID`。评论既追加进 `decision.jsonl`，又被下一次装配切进 bundle 回喂给模型。`CONFIRMED` 清关卡并推进；`REVISE` 回退对应细粒度状态；`VOID <CH>` 记 `void_record` 且章节 `void → planned`（最小语义，不级联 / 不过期检测）。
- **`mojian status [--path <dir>]`**：在 SOP 阶段之外，追加显示「卡在哪个关卡 / 等待哪种判定」。

生成命令默认 `claude`，可经 `MOJIAN_CLAUDE_CMD` 环境变量注入替换为假命令（测试端到端跑通 `run → decide → run` 而不触达真实 claude、不花 token）。机器状态与 `generation.jsonl` / `decision.jsonl` 日志落客户端（按 `project_id`），项目目录不含机器状态。本迭代仍用占位 SPEC（一个可跑的 `brief_drafting` 步 + `brief` 关卡）只验机制通路，真实创作提示词与多关卡链路待后续迭代。
```

**原因：** run/decide/status 从桩转真逻辑是本迭代用户可见的核心能力，product.md 记录「已落地」必须反映生成闭环机制通路已打通。
**对应任务：** TASK-002 / TASK-003 / TASK-004 / TASK-005 / TASK-006

### 章节「已落地功能」· ITER-001 小节：修正现已过时的桩描述

**原文：**

```
- **`mojian run` / `mojian decide`**：本阶段为桩（打印「stub，将在 ITER-002 实现」并以 exit 0 正常退出），创作生成循环尚未接入。
```

**新文：**

```
- **`mojian run` / `mojian decide`**：ITER-001 阶段为桩；已于 ITER-002 转为真逻辑（见下节「生成闭环」）。
```

**原因：** product.md 记录当前已落地能力，保留「run/decide 是桩」会与新增的 ITER-002 小节矛盾、误导读者。
**对应任务：** TASK-006

### 章节「尚未落地」：收敛已落地部分，保留真正未落地项

**原文：**

```
创作生成循环（SDK 调用、切片装配、人机决定循环、客观检查器、VOID / 过期检测、日志文件写入、题材 SPEC 变体、统计产出）——见 `docs/tech-design/engine.md`，规划自 ITER-002 起。
```

**新文：**

```
创作生成循环的机制通路已于 ITER-002 落地（见上）。尚未落地：客观检查器（字数 / 对话占比 / 结构 + `check.jsonl`）、VOID 圣经级联与输入过期检测、真实 `claude --output-format json` schema 收口（本迭代只跑 mock）、真实 SOP 提示词与多关卡链路（当前为占位 SPEC）、题材 SPEC 变体、统计产出——见 `docs/tech-design/engine.md`。
```

**原因：** SDK 调用 / 切片装配 / 人机决定循环 / 日志文件写入已落地，需从「尚未落地」移出；客观检查器、VOID 级联/过期检测、真实 claude schema、真实 SOP 提示词仍未落地，需明确保留。
**对应任务：** TASK-002~006（落地项） + 裁决① / 裁决③（保留项边界）

## 对项目级技术设计基线的修改建议

### docs/tech-design/overview.md · 章节「技术栈基线」：追加 serde_json 一行

**原文（表头 + 末行）：**

```
## 技术栈基线（ITER-001 落地）
...
| 其余 | `uuid` / `time` / `anyhow` / `thiserror` | project_id / 时间戳 / 应用层与库错误 |
```

**新文：**

```
## 技术栈基线（ITER-001 落地，ITER-002 增补）
...
| 其余 | `uuid` / `time` / `anyhow` / `thiserror` | project_id / 时间戳 / 应用层与库错误 |
| JSON 解析 / JSONL 序列化（ITER-002） | `serde_json` | 解析 `claude --output-format json` 的 stdout；generation/decision 事件逐行序列化写 JSONL；serde 生态标准件，与既有 `serde` 协作 |
```

**原因：** `serde_json` 是 ITER-002 唯一新增依赖，锁定于 workspace 依赖基线，需进项目级技术栈基线表。
**对应任务：** TASK-001

### docs/tech-design/engine.md · 章节「切片装配器 + 输入契约 manifest」：措辞对齐 TOML sidecar

**原文：**

```
每个 SOP 步骤在其 SPEC 里声明**输入契约**（manifest），用**符号引用**而非死路径：

```yaml
# skeleton agent 的输入契约（写在 agent frontmatter 或 sidecar）
```

**新文：**

```
每个 SOP 步骤在其 SPEC 里声明**输入契约**（manifest），用**符号引用**而非死路径。落地形态为**部署 SPEC 内的 TOML sidecar**（`<agent>.manifest.toml`，与 agent 正文物理分离：Rust 读契约、`claude` 读提示词）；下方 YAML 为等价示意，语义（符号引用 / `write:` → `write_scope`）一致：

```yaml
# skeleton agent 的输入契约（落地为 TOML sidecar；此处 YAML 仅为等价示意）
```

**原因：** ITER-002 实现取 TOML sidecar（省 `serde_yaml` 依赖），engine.md 原文「frontmatter 或 sidecar」已含该选项，归档时把示意措辞对齐为落地形态，避免读者误以为 YAML frontmatter 是承诺格式。
**对应任务：** TASK-004（设计选型 3-A）

### docs/tech-design/engine.md · 章节「SDK 调用：Rust → 无头 `claude` 子进程」：补生成命令可注入替换

**原文（该节末尾列表最后一项之后）：**

```
- 备选（不采用）：Rust 直接打 Anthropic API —— 丢了工具循环，得自己实现 Read/Write。
```

**新文：**

```
- 备选（不采用）：Rust 直接打 Anthropic API —— 丢了工具循环，得自己实现 Read/Write。
- **基础命令可注入（ITER-002 落地）**：`claude` 为默认基础命令，可经 `MOJIAN_CLAUDE_CMD` 环境变量替换为假命令；配合 `GenerationRunner` trait（core 单测用 `FakeRunner` 不起进程），使 QA 能在不触达真实 `claude`、不花 token 前提下端到端验证全链路。
```

**原因：** 「外部生成命令可替换、测试注入 mock」是本迭代可测试性硬约束的落地机制，属项目级设计基线应记录的接缝。
**对应任务：** TASK-003 / TASK-006（设计选型 1-A）

## 知会（不改文件，供确认）

- **关卡持久化落 `cursors` JSON**：SOP① 顺序关卡的 pending 标记借 `project_state.cursors` JSON 承载，未新增列，`SCHEMA_VERSION` 保持 1。属实现细节，与 storage.md / engine.md 基线不冲突；ITER-003 若关卡语义变复杂可提升为独立列。本轮不改 storage.md（无迁移、无表结构变化）。

## 不归档的 done 任务

| TASK | 原因 |
|------|------|
| TASK-001 | error 变体扩展为内部实现细节，不入 PRD；其 `serde_json` 依赖已由 overview.md 技术栈基线一行覆盖，不单列 |
| TASK-002 | `generation.jsonl` / `decision.jsonl` 写入器为 engine.md 既有设计（storage.md「六」日志走文件）的实现，产物已由 product.md「生成闭环」小节体现，无需改基线 |
| TASK-005 | `next_action` / `apply_*` / state DB 读写为 engine.md / storage.md 既定语义的实现级落地，无基线偏离；关卡入 cursors 已在「知会」列出 |

（TASK-003 / TASK-004 / TASK-006 的基线级偏离已在上方 tech-design 修改建议中归档。）

## 应修文案（供人工决定，本 agent 不改代码）

- `crates/mojian-cli/src/main.rs` 子命令 help 文案与模块 doc 注释仍写「桩，将在 ITER-002 实现」（第 4 / 26 / 28 行）。因 `main.rs` 在 TASK-006 禁改清单未被更新，现已过时（run/decide 已是真逻辑）。**建议**：本迭代内以纯文案修正（不涉逻辑，属 CLAUDE.md「注释/格式调整」例外）更新，或另开小 issue 于后续迭代处理。请人工裁决。archivist-agent 不改业务代码。

## 协议复核建议（本轮变更是否导致框架文档需要精简）

- 无需精简。本迭代为业务代码迭代，未触及 `.esr_harnass/protocol/builder_driver.md` 或各 agent 合同；框架文档无重复 / 过时 / 可合并内容引入。

## 待确认项

- [x] product.md 3 项修改建议是否同意？（新增「生成闭环」小节 / 修正 ITER-001 桩描述 / 收敛「尚未落地」）—— 同意，已应用
- [x] tech-design 3 项修改建议是否同意？（overview.md 加 serde_json 行 / engine.md manifest 对齐 TOML sidecar / engine.md 补 MOJIAN_CLAUDE_CMD 可注入）—— 同意，已应用
- [x] 关卡入 cursors JSON 的「知会」是否认可（本轮不改 storage.md）？—— 认可，storage.md 不动
- [x] 不归档清单（TASK-001/002/005）是否合理？—— 合理
- [x] main.rs 过时 help 文案：本迭代内修正 or 另开 issue？—— 本迭代内修（由 driver 在 Git 收口前更正，属 CLAUDE.md 文案例外，不另开 issue）
