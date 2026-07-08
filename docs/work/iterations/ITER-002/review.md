# Review — ITER-002

## TASK-001 — 2026-07-08 — ✅ 通过

dev 环境：Rust workspace（devops.md Build Verification 口径），无外部服务；`cargo build --workspace` EXIT=0

QA Verification：
  [x] `cargo build --workspace` 退出码 0 — 命令：`cargo build --workspace`；响应：`Finished dev profile ... target(s)`，EXIT=0
  [x] serde_json 进依赖图 — 命令：`cargo tree -p mojian-core | grep serde_json`；响应：`├── serde_json v1.0.150`
  [x] 未引入被禁依赖 — 命令：`cargo tree -p mojian-core | grep -E 'serde_yaml|tokio|reqwest'`；响应：无输出（grep exit 1）

附加验证（Builder Exit 回归口）：
  [x] `cargo test -p mojian-core --lib` — `test result: ok. 9 passed; 0 failed`

运行结论：
  所有 QA Verification 通过 ✓（3/3），守住 overview.md「零 token 花费面」——唯一新增依赖 serde_json，无 serde_yaml/tokio/reqwest

## TASK-002 — 2026-07-08 — ✅ 通过

dev 环境：Rust workspace（devops.md Build Verification 口径），无外部服务；MOJIAN_HOME 指向 mktemp 隔离目录；`cargo build --workspace` EXIT=0

QA Verification：
  [x] `cargo test -p mojian-core --test log_jsonl` 退出码 0（集成测试真实写盘到隔离 data_dir） — 命令：`MOJIAN_HOME=<临时目录> cargo test -p mojian-core --test log_jsonl`；响应：`test result: ok. 3 passed; 0 failed`，INT_EXIT=0
  [x] 连续两次 append_generation → generation.jsonl 实为两行、每行可 serde_json 反序列化、第一行未被第二次写入改动 — test `append_generation_is_append_only_two_lines`；响应：断言 `path.exists()` / `lines.len()==2` / `content.starts_with(first_line)` / 两行反序列化字段还原（step brief_drafting/vision_drafting，hash-1/hash-2）均 ok
  [x] 两条不同 gate 的 DecisionEvent → `read_decision_comments(project_id, "brief", None)` 只返回 gate=="brief" 评论 — test `read_decision_comments_filters_by_gate`；响应：`assert_eq!(comments, vec!["收紧题材定位"])` ok（gate=="vision" 的「更换金手指」被过滤）

附加验证（Builder Exit 回归口）：
  [x] `cargo build --workspace` — `Finished dev profile`，BUILD_EXIT=0
  [x] log 模块单测 `cargo test -p mojian-core --lib log` — `test result: ok. 5 passed; 0 failed`（pick_comment gate/target/空 comment 边界 + generation 单行 JSON + decision 往返省略空 Option）

运行结论：
  所有 QA Verification 通过 ✓（3/3）；JSONL 只增不改、每行合法 JSON、read_decision_comments 按 gate/target 往返均以真实磁盘断言验证
