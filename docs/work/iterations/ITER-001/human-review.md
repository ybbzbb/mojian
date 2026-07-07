# Human Review — ITER-001

## Round 1 — 2026-07-07 [requirements-agent review decision]

阶段: review_done
变更类型: feature
影响面: large
风险信号:
- 全新 Cargo workspace 奠基，后续所有迭代依赖此结构
- 客户端中央 DB schema 一次性落地 12 表，跨两个作用域（客户端 / 项目）
- 有两处落点未定（客户端数据目录位置、SPEC 主副本初始内容）

建议: human_review

（review_policy = human_required，且影响面 = large → 强制等待人工）

---

## Round 1 — 2026-07-07 [requirements-agent output]

GAP 摘要：功能 GAP 9 项 | 架构 GAP 4 项 | 可行性 ✅ | 里程碑「运行环境就位」
product.md 变更建议：无

等待操作：在下方 [human feedback] 块填写决定，或在 issue #9 / #10 回复，然后重触发 requirements-agent
- 决定 = "修订"     → 生成 revision 2
- 决定 = "CONFIRMED" → 终审检查，通过则自动进入 design_ready

请特别就 requirements.md「待确认项」的 4 个问题给出意见（数据目录位置 / SPEC 主副本内容 / 初始 phase / 桩命令行为）。

---

## Round 1 — 2026-07-07 [human feedback]

### 意见

4 个待确认项的决定：

1. **客户端数据目录位置**：交给 design-agent 定，用平台标准目录（Linux XDG / mac Application Support），默认 `~/.mojian/`。
2. **SPEC 主副本初始内容**：本迭代放**占位骨架**，仅验证「部署 + hash 覆盖」通路；真实提示词等 SOP #5/#6/#7 定稿后再填。
3. **新建项目初始 phase**：`mojian status` 首次输出 `style_sampling`（SOP① 首阶段）。
4. **桩命令行为**：`mojian run` / `mojian decide` 打印「stub，将在 ITER-002 实现」并正常退出，可接受。

需求边界清晰，其余需求（REQ-001~014）无异议。

### 决定
CONFIRMED

---

## Round 2 — 2026-07-07 [requirements-agent output] (final check)

终审通过：收到 Round 1 CONFIRMED，最终检查全部通过：
- 需求清晰度：REQ-001~014 无 TBD/暂定/待定/含糊量词。
- 范围明确：「目标」与「不在范围内」清晰，REQ 之间无矛盾。
- 变更清单一致：gap-analysis.md 的 product.md 变更建议为「无」，与 requirements.md 一致。
- 无新待确认项：原 4 项待确认已由 Round 1 [human feedback] 决定，并入 requirements.md 作为已确认约束/边界，全部勾选。

4 项决定并入结果：
1. 客户端数据目录位置 → 约束节：默认 `~/.mojian/`，平台标准目录细节交 design-agent。
2. SPEC 主副本初始内容 → REQ-011：本迭代放占位骨架，仅验证部署 + hash 覆盖通路。
3. 新建项目初始 phase → 约束节：`style_sampling`。
4. 桩命令行为 → REQ-010：打印「stub，将在 ITER-002 实现」并 exit 0 正常退出。

phase: review_done → design_ready
requirements.md / gap-analysis.md status → confirmed

---

## Round 3 — 2026-07-07 [design-agent review decision]

阶段: design_review
变更类型: feature
影响面: large
风险信号:
- 一次性 12 表 schema 落地，后续迁移演进走自研 schema_meta 迁移器
- 跨作用域一致性：机器状态在客户端 DB、SPEC 部署在项目文件系统
- 首个 Cargo workspace 与技术栈选型奠基，被后续所有迭代继承（锁定成本高）
- 三处委派落点由 design-agent 判断（数据目录解析顺序 / SPEC 主副本=部署布局 / 项目缓存 hash 实时重算）

建议: human_review

（review_policy = human_required，且影响面 = large → 强制等待人工）

---

## Round 3 — 2026-07-07 [design-agent output]

设计摘要：采用 rusqlite(bundled) + clap4(derive) + directories + serde/toml + blake3 + uuid/time/anyhow/thiserror/include_dir；workspace = mojian-core(库) + mojian-cli(二进制 mojian)；CLI 面 new/status 实现 + run/decide 桩；12 表建库 + schema_meta 迁移器；SPEC 主副本 1:1 部署 + blake3 tree hash 覆盖（选项 A）
PRD 影响：4 项细化（数据目录解析 / 主副本=部署布局 / 项目缓存 hash 定义 / 选型基线）；DevOps 影响：3 项构建期要求（cargo + C 编译器 / build 验证命令 / 单二进制运行）

等待操作：在下方 [human feedback] 块填写决定，或在 issue #9 / #10 回复，然后重触发 design-agent
- 决定 = "修订"     → 生成 revision 4
- 决定 = "CONFIRMED" → 终审检查，通过则自动进入 planning_ready

请特别就 tech-design.md「待确认项」的三处委派落点给出意见（数据目录解析顺序 / SPEC 主副本=部署布局 / 项目缓存 hash 实时重算）。

---

## Round 3 — 2026-07-07 [human feedback]

### 意见

选型与 3 个委派落点全部认可：
1. 数据目录解析顺序 `MOJIAN_HOME → 平台标准目录 → ~/.mojian/ 兜底` —— 同意（MOJIAN_HOME 便于隔离测试）。
2. SPEC 主副本布局 = 部署布局（1:1，SOP 包置 prompts/ 下）—— 同意。
3. 项目缓存 hash 实时重算部署树（不存 hash 标记）—— 同意。

选型（rusqlite bundled / clap4 / directories / serde+toml / blake3 / include_dir + 自研 schema_meta 迁移器）无异议。PRD/DevOps 影响留待归档处理。

### 决定
CONFIRMED

---

## Round 3 — 2026-07-07 [design-agent output] (final check)

终审通过：收到 Round 3 CONFIRMED，最终检查全部通过：
- 选型决策：采用方案（rusqlite bundled / clap4 / directories / serde+toml / blake3 / include_dir + 自研 schema_meta 迁移器）明确、无未决项；三处委派落点（数据目录解析顺序 / SPEC 主副本=部署布局 1:1 / 项目缓存 hash 实时重算）已由人工逐项认可。
- 接口完备：REQ-001~014 全部映射到具体 CLI 命令契约 / 模块 / 数据模型变更（见「涉及模块」「API 变更」「数据模型变更」）。
- 风险声明：「依赖与风险」非空——4 条已知风险（12 表迁移演进 / 跨作用域一致性 / workspace 奠基锁定 / 委派落点）均附缓解措施。
- 与 PRD 不冲突：设计与项目级技术设计基线（overview/storage/engine）一致，「PRD 影响」为 4 项兼容性实现级细化，无冲突项。
- DevOps 兼容：不新增端口/账号/常驻服务；仅构建期要求（cargo + C 编译器），列入「DevOps 影响」留待归档。
- 无新待确认项：tech-design.md「待确认项」7 项全部勾选（3 处委派落点 Round 3 确认，PRD/DevOps 影响 Round 3 决定留待归档）。

phase: design_review → planning_ready
tech-design.md status → confirmed

下一步：planning-agent 自动接手，拆分任务。
