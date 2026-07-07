# Review Log — ITER-001

## TASK-001 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（无需启动服务，CLI 项目直接跑 cargo/二进制）

QA Verification：
  [x] cargo build --workspace 退出码 0，target/debug/mojian 存在 — 命令：`cargo build --workspace`；退出码 0；`ls -l target/debug/mojian` → 444864 字节，可执行位存在
  [x] cargo test -p mojian-core 退出码 0，0 failed（含 paths 解析测试） — 命令：`MOJIAN_HOME=$(mktemp -d) cargo test -p mojian-core`；退出码 0；tests/paths.rs `mojian_home_overrides_data_dir_and_helpers ... ok`（1 passed; 0 failed）
  [x] 最小骨架可运行不 panic — 命令：`MOJIAN_HOME=$(mktemp -d) target/debug/mojian`；退出码 0
  [x] cargo check --workspace 退出码 0，无 error — 命令：`cargo check --workspace`；退出码 0；无 error 输出

运行结论：
  所有 QA Verification 通过 ✓（4/4，均在隔离 MOJIAN_HOME 临时目录下真跑，未污染真实 ~/.mojian）

## TASK-002 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（库/CLI 编译+单测型验收，无需启动服务）；隔离 MOJIAN_HOME 临时目录，未污染真实 ~/.mojian

QA Verification：
  [x] cargo build --workspace 退出码 0 — 命令：`cargo build --workspace`；输出 `Finished dev profile`；退出码 0
  [x] cargo test -p mojian-core domain 退出码 0，0 failed（含三枚举 DB 文本往返测试与非法输入测试） — 命令：`cargo test -p mojian-core domain`；`test result: ok. 9 passed; 0 failed`；9 个用例：chapter_state/extract_status/sop_phase 各 as_db_str_matches_mapping_table + roundtrip_every_variant + unknown_value_is_err；退出码 0

REQ-005 逐字一致抽查（枚举变体 ↔ DB 文本 ↔ tech-design 映射表/ naming.md）：
  [x] SopPhase 10 变体：StyleSampling/StyleExtracting/BriefDrafting/VisionDrafting/BibleBuilding/BibleCheck/BibleVerify/OutlineExpand/OutlineVerify/Writing → style_sampling/…/writing，与 tech-design.md L174 逐字一致
  [x] ChapterState 7 变体：Planned/SkeletonDrafting/SkeletonReview/ProseDrafting/ProseReview/Approved/Void → planned/…/void，与 tech-design.md L175 及 naming.md 章节状态命名逐字一致
  [x] ExtractStatus 3 变体：Pending/Extracting/Extracted → pending/extracting/extracted，与 tech-design.md L176 逐字一致

运行结论：
  所有 QA Verification 通过 ✓（2/2），REQ-005 代码与文档单一事实源无两套叫法

## TASK-003 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（库/CLI 编译+集成测试型验收，无需启动服务）；隔离 MOJIAN_HOME/TMPDIR 临时目录，未污染真实 ~/.mojian

QA Verification：
  [x] cargo build --workspace 退出码 0 — 命令：`cargo build --workspace`；输出 `Finished dev profile`；退出码 0
  [x] cargo test -p mojian-core db 退出码 0，0 failed — 命令：`cargo test -p mojian-core db`；退出码 0；tests/db.rs `running 3 tests` → `test result: ok. 3 passed; 0 failed`：
        - db_fresh_creates_all_twelve_tables_and_stamps_version（新建库后 12 具名表齐全 + schema_meta.schema_version==1，且恰好 12 张业务表）... ok
        - db_reopening_same_path_is_idempotent（同路径二次 open_central_db 幂等：不重复建表、版本仍 1、schema_meta 仅 1 行）... ok
        - db_foreign_keys_pragma_is_enabled（PRAGMA foreign_keys 返回 1）... ok

运行结论：
  所有 QA Verification 通过 ✓（2/2）。任务要求的抽查项（12 表齐全 / schema_version=1 / 二次 open 幂等 / PRAGMA foreign_keys=ON）均由 tests/db.rs 三个集成测试断言并真跑通过。

## TASK-004 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（库/CLI 编译+集成测试型验收，无需启动服务）；隔离 MOJIAN_HOME 临时目录，未污染真实 ~/.mojian

QA Verification：
  [x] cargo build --workspace 退出码 0 — 命令：`cargo build --workspace`；退出码 0；输出 `Finished dev profile`
  [x] cargo test -p mojian-core project 退出码 0，0 failed — 命令：`cargo test -p mojian-core project`；退出码 0；tests/project.rs `running 4 tests` → `test result: ok. 4 passed; 0 failed`：
        - project_register_then_load_returns_style_sampling（register→load 得 SopPhase::StyleSampling，且存入 path 为绝对路径）... ok
        - project_manifest_write_read_roundtrip（write_manifest → read_manifest 往返 project_id/spec_version 一致）... ok
        - project_load_unknown_is_err（无此 project 返回 CoreError::Db）... ok
        - project_read_missing_manifest_is_err（缺 mojian.toml 返回 CoreError::Io）... ok

运行结论：
  所有 QA Verification 通过 ✓（2/2）。任务要求的断言（register→load 初值 style_sampling / mojian.toml 读写往返）均由 tests/project.rs 集成测试真跑通过。

## TASK-005 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（纯 workspace 库/CLI，无服务端；隔离 MOJIAN_HOME=mktemp 临时目录，未污染真实 ~/.mojian）

QA Verification：
  [x] cargo build --workspace 退出码 0 — 命令：`cargo build --workspace`；退出码 0；`Finished dev profile ... target(s)`
  [x] cargo test -p mojian-core spec 退出码 0，0 failed（含四类断言） — 命令：`cargo test -p mojian-core spec`（退出码 0）+ `cargo test -p mojian-core --test spec`（退出码 0，跑全 6 用例覆盖四类断言）；tests/spec.rs `test result: ok. 6 passed; 0 failed`：
        - tree_hash 确定性/内容敏感 → tree_hash_is_order_independent_and_content_sensitive ... ok
        - deploy 生成部署目标（含无 spec.toml + hash==authoritative） → deploy_places_targets_without_spec_toml_and_hash_matches_authoritative ... ok
        - drift 覆盖还原 → sync_if_drifted_restores_tampered_file ... ok / sync_if_drifted_restores_deleted_file ... ok
        - 无 drift 不写 → sync_if_drifted_no_drift_does_not_overwrite ... ok
        - 主副本 bootstrap + version 落地 → bootstrap_writes_master_tree_and_version ... ok

运行结论：
  所有 QA Verification 通过 ✓（2/2）。四类断言在 tests/spec.rs 6 个集成测试用例中真实覆盖并全绿。
  备注：字面命令 `cargo test -p mojian-core spec` 中 `spec` 是名字过滤，仅匹配 1 个名字含 "spec" 的用例（其余按名字被 filtered out），故补跑 `--test spec` 跑完整 spec 集成测试二进制，确认四类断言全部真实执行且通过。

## TASK-006 — 2026-07-07 — ✅ 通过

dev 环境：Rust 工具链 cargo 1.96.1（纯 workspace 库/CLI，无服务端）。真实二进制 `target/debug/mojian`；全程 `MOJIAN_HOME=$(mktemp -d)` + `PROJ="$(mktemp -d)/mybook"` 隔离临时目录，未污染真实 ~/.mojian。构建：`cargo build --workspace` 退出码 0。

QA Verification：
  [x] mojian new 退出码 0 + UUID/绝对 path/style_sampling — 命令：`mojian new "$PROJ"`；exit 0；stdout `project_id: d236332c-e57a-488c-984c-9ee35a31b5bf` / `path: .../mybook` / `phase: style_sampling`
  [x] central.db + 表数 ≥12 + schema_version=1 — `test -f central.db` OK；`SELECT count(*) ... type='table'` = 13（≥12）；`SELECT schema_version FROM schema_meta` = 1
  [x] mojian.toml 含 project_id/spec_version — `test -f mojian.toml` OK；grep 命中 `project_id = "d236332c..."` 与 `spec_version = "0.0.1-skeleton"`
  [x] SPEC 已部署且无 spec.toml — CLAUDE.md / prompts/sop-1-style / .claude/agents 均 test 成立（另有 prompts/sop-2-bible、sop-3-writing、.claude/skills）；`test -e spec.toml` 不成立
  [x] REQ-014 spec 两列非空 — `SELECT spec_version, spec_hash FROM project` = `0.0.1-skeleton|fd827d90...5104`，BOTH_NONEMPTY
  [x] mojian status --path 退出码 0 + style_sampling — 命令：`mojian status --path "$PROJ"`；exit 0；stdout `project: mybook` / `phase: style_sampling`
  [x] REQ-013 hash 覆盖 — `echo tampered >> CLAUDE.md`（count=1）→ `mojian status`（exit 0）→ `grep -c tampered CLAUDE.md` = 0（重部署覆盖还原）
  [x] 桩命令 run/decide — `mojian run` exit 0 stdout `stub，将在 ITER-002 实现`；`mojian decide CH-001 CONFIRMED` exit 0 stdout 同提示（忽略尾随参数）
  [x] 错误路径 非 mojian 项目 — `mojian status --path <空目录>` exit 1；输出 `错误：非 mojian 项目：目录下无 mojian.toml (...)`
  [x] 重复初始化拒绝 — 对已初始化 `$PROJ` 再 `mojian new "$PROJ"` exit 1；输出 `错误：目录已是 mojian 项目（存在 mojian.toml），拒绝重复初始化：...`

独立复核：
  cargo build --workspace 退出码 0；cargo test --workspace 全绿 —— cli.rs 5 / lib.rs 9 / db.rs 3 / paths.rs 1 / project.rs 4 / spec.rs 6 = 28 passed，0 failed。

运行结论：
  所有 QA Verification 通过 ✓（10/10）。真实二进制端到端收口验证：mojian new 建目录+中央 DB 登记+SPEC 部署+回填 project 行 spec 列+写 manifest，mojian status 读 manifest+打开时 hash 覆盖还原+读回 phase，run/decide 桩，错误路径与重复初始化拒绝均符合期望。

---

## QA 验收完成 — 2026-07-07

完成任务：6 个
取消任务：0 个
跳过任务：0 个
总计：6 个

交付摘要（每个 done 任务一行）：
- TASK-001: workspace 骨架 + 依赖基线 + paths/error — QA Verification 2/2 ✓
- TASK-002: domain 三枚举 + DB 文本互转 — QA Verification 2/2 ✓
- TASK-003: 中央 DB schema v1（12 表）+ 迁移器 + open_central_db — QA Verification 2/2 ✓
- TASK-004: project 注册 + manifest 读写 — QA Verification 2/2 ✓
- TASK-005: SPEC 主副本 bootstrap + deploy + hash 漂移覆盖 — QA Verification 2/2 ✓
- TASK-006: CLI 命令 new/status + run/decide 桩（端到端收口）— QA Verification 10/10 ✓
