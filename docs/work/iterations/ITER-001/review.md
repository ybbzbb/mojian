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
