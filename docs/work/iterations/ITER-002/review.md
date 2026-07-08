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

## TASK-003 — 2026-07-08 — ✅ 通过

dev 环境：Rust workspace（devops.md Build Verification 口径），无外部服务；MOJIAN_HOME 指向 mktemp 隔离目录；`cargo build --workspace` EXIT=0

QA Verification：
  [x] `cargo test -p mojian-core --test sdk_runner` 退出码 0（集成测试用 MOJIAN_CLAUDE_CMD 指向测试内生成的假命令脚本，ClaudeCliRunner 真实 spawn 该子进程验证「外部命令可替换」硬约束，不触达真实 claude） — 命令：`cargo test -p mojian-core --test sdk_runner`；响应：`test result: ok. 2 passed; 0 failed`，SDK_RUNNER_EXIT=0（tests/sdk_runner.rs 用 write_fake_command 写 0o755 sh 脚本，ClaudeCliRunner::new(temp_dir).run 真实 spawn）
  [x] 假命令输出固定 JSON 时，run 返回的 SdkResponse.result 等于假命令产出文本，且 cost/usage_in/usage_out 被正确解析 — test `spawns_injected_command_and_parses_fixed_json`；响应：假脚本输出 `{"result":"占位创作物 brief","total_cost_usd":0.0123,"usage":{"input_tokens":128,"output_tokens":256}}`，断言 `result=="占位创作物 brief"` / `cost==Some(0.0123)` / `usage_in==Some(128)` / `usage_out==Some(256)` 全 ok
  [x] 假命令以非 0 退出码结束时，run 返回 Err（SubprocessFailed 变体）不 panic — test `non_zero_exit_returns_subprocess_failed_without_panic`；响应：假脚本 `exit 3` + stderr `boom`，`unwrap_err` 匹配 `CoreError::SubprocessFailed { code: Some(3), stderr.contains("boom") }` ok

附加验证（FakeRunner trait 注入 + SdkResponse 容错回归口）：
  [x] `cargo test -p mojian-core --lib sdk` — `test result: ok. 4 passed; 0 failed`：`fake_runner_returns_injected_response_without_spawn`（FakeRunner 实现 GenerationRunner，不 spawn 进程）、`sdk_response_parses_full_json`、`sdk_response_tolerates_missing_cost_and_usage`（缺 total_cost_usd/usage → Option None 不报错）、`sdk_response_tolerates_partial_usage`
  [x] `cargo build --workspace` — `Finished dev profile`，BUILD_WORKSPACE_EXIT=0

运行结论：
  所有 QA Verification 通过 ✓（3/3）；ClaudeCliRunner 经 MOJIAN_CLAUDE_CMD 真实 spawn 假命令验证外部命令可替换 + JSON 解析路径，非 0 退出返回 SubprocessFailed 不 panic；FakeRunner trait 注入与 SdkResponse total_cost_usd/usage 容错解析经 lib 单测覆盖
