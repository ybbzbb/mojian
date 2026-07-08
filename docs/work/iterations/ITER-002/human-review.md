# Human Review — ITER-002

来源：issue#11 + issue#12（里程碑「生成闭环」）

---

## Round 1 — 2026-07-07 [requirements-agent review decision]

阶段: review_done
变更类型: feature
影响面: large
风险信号:
- 核心生成循环，跨多模块（manifest 解析 / 切片装配 / 无头 claude 子进程 / 三条 JSONL 日志 / 状态机推进行为 / CLI run/decide）——影响面 large。
- 客观检查器边界不明：engine.md 与 issue#12 的 run 循环含「客观检查」一步，但用户本迭代范围描述省略之；需人工拍板是否纳入（待确认项①）。
- 占位 SPEC 下 `run` 的可推进深度未定——「机制通路」的最小验收深度需人工给期望（待确认项②）。
- SDK 外部命令可替换（测试注入假命令）是可测试性硬约束，注入机制的落地方案需 design 阶段确定。

建议: human_review（review_policy = human_required，强制人工确认）

---

## Round 1 — 2026-07-07 [requirements-agent output]

GAP 摘要：功能 GAP 9 项 | 架构 GAP 8 项 | 可行性 ✅（机制通路层面，边界待人工拍板）| 无选项对比（选型交 design）
product.md 变更建议：无（归档阶段处理）
待确认项：3 项开放（客观检查器是否纳入 / run 可推进深度 / VOID 本迭代深度）

等待操作：在下方 [human feedback] 块填写决定，然后重触发 requirements-agent
- 决定 = "修订"     → 生成 revision 2
- 决定 = "CONFIRMED" → 终审检查，通过则自动进入 design_ready

（source 为 issue：也可在 issue #11 / #12 评论区回复意见或 "CONFIRMED"，agent 会自动同步）

---

## Round 1 — 2026-07-07 [human feedback]

### 意见

3 项待确认项裁决（与既定迭代划分一致）：

1. **客观检查器 + check.jsonl**：**排除本迭代**——属 IMPL-5，归 ITER-003。run 循环里「客观检查」这一步本迭代留位不实现（生成后直接进关卡/落库）。
2. **机制通路最小验收深度**：串通一次 `run → decide → run`——run 装配→调（可注入的 mock）SDK→写 generation.jsonl→推进到人工关卡即停；decide 写 decision.jsonl→推进状态机；人评论被下一次装配切进 bundle。用注入的假命令端到端测，不要求真实创作产物。
3. **decide VOID 深度**：仅记录 void_record + 最小状态推进（按指令 void→planned），**不做**圣经级联/过期检测——那是 IMPL-6，归 ITER-003。

其余需求（REQ-001~012）与硬约束（SDK 外部命令可替换、机器状态/三 JSONL 落客户端、沙箱 write_scope、命名对齐、占位 SPEC 只验机制）无异议。

### 决定
CONFIRMED

---

## Round 1 — 2026-07-07 [requirements-agent output] (final check)

终审通过。收到 Round 1 CONFIRMED + 3 项待确认项裁决，已并入 requirements.md「已确认边界」：

- 裁决①：客观检查器 + `check.jsonl` **排除本迭代**（归 ITER-003/IMPL-5）；run 循环「客观检查」步留位不实现。
- 裁决②：机制通路最小验收深度 = 串通一次 `run → decide → run`（可注入 mock SDK 端到端测，不要求真实创作产物，不要求多关卡链路）。
- 裁决③：`decide VOID` 仅记 `void_record` + `decision.jsonl` + 最小状态推进 `void→planned`，不做圣经级联 / 过期检测（归 ITER-003/IMPL-6）。

最终检查（全部通过）：
- 需求清晰度：REQ-001~012 无 TBD / 暂定 / 待定 / 含糊量词。
- 范围明确：目标与「不在范围内」清晰，3 项裁决消除边界模糊，REQ 间无矛盾。
- 变更清单一致：gap-analysis product.md 变更建议 = 无，与 requirements 引用一致。
- 无新待确认项：待确认项全部勾选。

requirements.md / gap-analysis.md status → confirmed
phase: review_done → design_ready

---

## Round 1 — 2026-07-07 [design-agent review decision]

阶段: design_review
变更类型: feature
影响面: large
风险信号:
- 核心生成循环，跨多新模块（sdk / context / log / engine / state + CLI run/decide/status）——影响面 large。
- 真实 `claude --output-format json` 字段 schema 本迭代未验证（只跑 mock，裁决②）；`SdkResponse` 解析按文档形态 + Option 容错，实测收口留待接真实 claude。
- manifest 格式偏离 engine.md 的 YAML frontmatter 示意，取 TOML sidecar（语义不变、依赖更省）——需人工确认可接受。
- 新增依赖 serde_json（本迭代唯一新依赖）。
- 关卡 pending 标记借 project_state.cursors JSON 承载（免迁移）——设计取舍需知会。

建议: human_review（review_policy = human_required，强制人工确认；且影响面 large）

---

## Round 1 — 2026-07-07 [design-agent output]

设计摘要：采用 GenerationRunner trait + MOJIAN_CLAUDE_CMD 双层 mock 注入 / std::process::Command / TOML sidecar manifest / serde_json / 手写符号解析 / run·decide 逻辑落 core engine；新增 4+1 个 core 模块（sdk/context/log/engine/state），CLI run·decide 从桩转实现、status 扩展显卡点；占位 SPEC 加一个可跑 brief_drafting 步 + brief 关卡，端到端跑通一次 run→decide→run。
新增 CLI 命令面：run（循环 next_action，撞关卡停）、decide <关卡> <判定> [目标] [--comment|--file]、status（扩展卡点显示）；无网络端点。
数据模型：无 DB schema 迁移（现有 12 表 + cursors JSON 足够）；generation/decision 两条 JSONL 落客户端 logs/{project_id}/。
PRD 影响：3 项（engine.md manifest 措辞对齐 TOML sidecar / overview.md 技术栈基线加 serde_json / 关卡标记入 cursors JSON 知会）——归档阶段处理。
DevOps 影响：无。

等待操作：在下方 [human feedback] 块填写决定，然后重触发 design-agent
- 决定 = "修订"     → 生成 revision 2
- 决定 = "CONFIRMED" → 终审检查，通过则自动进入 planning_ready

（source 为 issue：也可在 issue #11 / #12 评论区回复意见或 "CONFIRMED"，agent 会自动同步）

---

## Round 1 — 2026-07-07 [human feedback] (design)

### 意见

选型与 3 个取舍全部认可：
1. SDK 双层注入（GenerationRunner trait + MOJIAN_CLAUDE_CMD）—— 同意（core 单测 FakeRunner 不起进程、CLI E2E env 假命令，两层不花 token）。
2. manifest 取 TOML sidecar（`<agent>.manifest.toml`）—— 同意（engine.md 原文即「frontmatter 或 sidecar」，省 serde_yaml 依赖）。
3. 新增 serde_json —— 同意（解析 claude 输出 + 写 JSONL 必需，唯一新增依赖）。

「留位不留债」的接缝（检查器步位空过 / 切片 hash 只写不查 / SOP① 前置 phase 占位直进）处理得当。API 覆盖 REQ-001~012，风险评估充分。PRD 影响（engine.md manifest 措辞、overview.md 加 serde_json）留待归档。

### 决定
CONFIRMED


---

## Round 1 — 2026-07-07 [design-agent output] (final check)

终审通过。收到 Round 1 CONFIRMED（认可全部选型 + 3 个取舍 + 留位接缝处理），逐项最终检查通过：

- **选型决策**：选型 1-6 均有明确「采用」结论，无未决项（SDK 双层注入 `GenerationRunner` trait + `MOJIAN_CLAUDE_CMD` / `std::process::Command` / TOML sidecar manifest / `serde_json` / 手写符号解析 / run·decide 落 core engine）。
- **接口完备**：REQ-001~012 全部映射到 CLI 命令面（run/decide/status）与 core 库 API（sdk/context/log/engine/state）+ 数据落点。
- **风险声明**：「依赖与风险」非空——真实 claude JSON schema 留待 ITER-003 收口、SOP① 前置 phase 占位直进、章节级 VOID/REVISE 走 core 测、关卡标记入 cursors JSON，均诚实标注为留位不留债。
- **与 PRD 不冲突**：manifest 取 TOML sidecar 与 `engine.md`「frontmatter 或 sidecar」原文一致（sidecar 为其明列选项），语义（符号引用 / `write:` → `write_scope`）不变，非冲突；措辞对齐 + `overview.md` 加 `serde_json` 已列入「PRD 影响」，留待归档。
- **DevOps 兼容**：无新增端口 / 账号 / 服务 / 启动方式；`docs/devops.md` 无需变更。
- **无新待确认项**：4 项确认清单全部勾选。

tech-design.md status → confirmed
phase: design_review → planning_ready
