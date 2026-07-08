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

## TASK-004 — 2026-07-08 — ✅ 通过

dev 环境：Rust workspace（devops.md Build Verification 口径），无外部服务；MOJIAN_HOME 指向 mktemp 隔离目录；`cargo build --workspace` EXIT=0

QA Verification：
  [x] `cargo test -p mojian-core --test context_assemble` 退出码 0 — 命令：`cargo test -p mojian-core --test context_assemble`；响应：`test result: ok. 2 passed; 0 failed`，IT_EXIT=0（集成测试在隔离 MOJIAN_HOME + 唯一临时项目目录中 ensure_master→deploy_spec 部署占位 SPEC、open_central_db+register_project 种子最小 DB 行，真实调用 assemble_bundle 解析磁盘上的 `.claude/agents/brief-agent.manifest.toml`）
  [x] 断言 Bundle.agent 指向 brief-agent、write_scope 由 manifest write: 推导（非空、与白名单一致）、inputs 含被切片文件 content_hash — test `assemble_bundle_end_to_end_with_comment_feedback`；真实断言：`bundle.agent == ".claude/agents/brief-agent.md"`、`bundle.write_scope == vec!["creative/creative-brief.md"]` 且非空、`bundle.inputs.contains(slice_ref(CLAUDE.md 整文件).content_hash)` 与 `bundle.inputs.contains(slice_ref(brief-agent.md #inputs 段级).content_hash)` 均 ok；段级切片只取 `## inputs` 段不越界到 `## output`
  [x] decision.jsonl 写入 gate=="brief" 带 comment 记录后，assemble_bundle 的 Bundle.inputs 含该评论文本（REQ-011 回喂通路）— 同 test：append_decision 写入 `gate:brief / verdict:REVISE / comment:"把题材收紧到都市悬疑，弱化群像"`，断言 `bundle.inputs.contains(comment)` ok；反向 test `assemble_bundle_without_comments_has_no_feedback_block` 断言无 decision 时 inputs 仍含 `## input:` 切片但不含 `human comments` 回喂块

附加验证（context 单元测试回归口）：
  [x] `cargo test -p mojian-core context` — 25 passed; 0 failed（lib 单测：符号文法四类切分含 #anchor/:{params}/纯整文件、占位代入、段级 #anchor 抽取边界、整文件切片 hash 稳定），CTX_EXIT=0

运行结论：
  所有 QA Verification 通过 ✓（3/3）；assemble_bundle 端到端在隔离环境真跑，五字段 Bundle 装配 + write_scope 推导 + 段级/整文件 blake3 content_hash + decision.jsonl 人类评论回喂（REQ-011）均以真实磁盘断言验证；剩余 TASK-005/006 仍为 planned，迭代未关闭，phase 保持 building
