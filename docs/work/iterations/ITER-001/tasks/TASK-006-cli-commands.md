# TASK-006 CLI 命令 new / status + run / decide 桩

- iteration: ITER-001
- status: planned
- type: backend
- owner: builder-agent
- created: 2026-07-07
- updated: 2026-07-07

## Goal

在 `mojian-cli` 落地命令面（clap v4 derive）并把 core 能力编排成端到端命令：`mojian new <dir>`（建目录 + 中央 DB 登记 + SPEC 部署 + 写回 project 行 spec 列 + 写 `mojian.toml` + 打印 project_id/path/初始 phase）、`mojian status [--path <dir>]`（读 manifest + 打开时 hash 覆盖同步 + 读回并打印 SOP phase），`mojian run` / `mojian decide [args...]` 留桩。含把 `assets/spec/` 用 `include_dir!` 嵌入二进制并注入 core bootstrap。

## Allowed Files

- `crates/mojian-cli/src/main.rs`
- `crates/mojian-cli/src/commands/mod.rs`
- `crates/mojian-cli/src/commands/new.rs`
- `crates/mojian-cli/src/commands/status.rs`
- `crates/mojian-cli/src/commands/run.rs`
- `crates/mojian-cli/src/commands/decide.rs`
- `crates/mojian-cli/src/spec_assets.rs`（`include_dir!` 嵌入 `assets/spec/`）
- `crates/mojian-cli/Cargo.toml`（追加 `mojian-core`(path 依赖) 与本 crate 所需依赖行：`clap`(features `derive`) / `anyhow` / `include_dir`，均 workspace = true 或 path）
- `crates/mojian-cli/tests/**`
- 禁止：`crates/mojian-core/src/**`、`crates/mojian-cli/assets/**`

## Inputs

- 迭代 tech-design.md#API 变更（`mojian new` 有序 6 步、`mojian status` 有序 3 步、`run`/`decide` 桩行为）、#SPEC 部署 + hash 覆盖机制（打开时覆盖由 status 触发）、#选型 2（clap4 derive）、#涉及模块 `crates/mojian-cli/*` 布局
- requirements.md REQ-008 / REQ-009 / REQ-010 / REQ-012 / REQ-013 / REQ-014 + 约束「初始 phase = style_sampling」「桩命令打印『stub，将在 ITER-002 实现』并 exit 0」
- 依赖 core：`paths`（TASK-001）、`open_central_db`（TASK-003）、`register_project`/`load_project_state`/`update_project_spec`/`manifest`（TASK-004）、`spec::master`/`deploy`（TASK-005）

## Builder Exit Criteria

- [ ] clap（derive）定义 4 个子命令 `new` / `status` / `run` / `decide`，`--help` 与版本可用；`new` 接收必填 `<dir>`，`status` 接收可选 `--path <dir>`（默认当前工作目录），`decide` 接受并忽略尾随参数
- [ ] `spec_assets.rs` 用 `include_dir!("$CARGO_MANIFEST_DIR/assets/spec")` 嵌入占位主副本并传入 core bootstrap；首次运行时确保 `<data_dir>` 与 `<data_dir>/spec/` 落地
- [ ] `new.rs` 按 tech-design 有序 6 步实现：校验 `<dir>`（已含 `mojian.toml` → 报错非 0 退出）→ 确保数据目录/建库/主副本 bootstrap → `register_project`（事务，初始 `style_sampling`）→ `deploy_spec` 得 version/hash → `update_project_spec` 写回 project 行（REQ-014）→ `write_manifest`；stdout 输出 `project_id` + 绝对 `path` + 初始 phase `style_sampling`；exit 0
- [ ] `status.rs`：读目标目录 `mojian.toml` 取 `project_id`（缺失 → 报错「非 mojian 项目」非 0 退出）→ `sync_if_drifted` 打开时 hash 覆盖（不一致则重部署并 `update_project_spec`）→ `load_project_state` 读 `sop_phase` 并打印项目名 + 当前 SOP phase；exit 0
- [ ] `run.rs` / `decide.rs`：打印 `stub，将在 ITER-002 实现` 并 exit 0（`decide` 忽略尾随参数）
- [ ] 集成测试（`crates/mojian-cli/tests/`，用 `MOJIAN_HOME` 指向临时目录）覆盖 new→status 正常路径与「非 mojian 项目」错误路径，均通过
- [ ] `cargo check --workspace` / `cargo build --workspace` 0 error；命名遵循 docs/naming.md

## QA Verification

前置：`export MOJIAN_HOME=$(mktemp -d)`；`export PROJ="$(mktemp -d)/mybook"`；二进制路径 `target/debug/mojian`（先 `cargo build --workspace`，退出码 0）。

- [ ] `target/debug/mojian new "$PROJ"` 退出码 0，stdout 含 UUID 形式 project_id、`$PROJ` 绝对路径、`style_sampling`
- [ ] `test -f "$MOJIAN_HOME/central.db"` 成立；`sqlite3 "$MOJIAN_HOME/central.db" "SELECT count(*) FROM sqlite_master WHERE type='table'"` 返回 ≥ 12；`sqlite3 "$MOJIAN_HOME/central.db" "SELECT schema_version FROM schema_meta"` 返回 `1`
- [ ] `test -f "$PROJ/mojian.toml"` 成立；`grep project_id "$PROJ/mojian.toml"` 命中；`grep spec_version "$PROJ/mojian.toml"` 命中
- [ ] SPEC 已部署：`test -f "$PROJ/CLAUDE.md"` 且 `test -d "$PROJ/prompts/sop-1-style"` 且 `test -d "$PROJ/.claude/agents"` 均成立；`test -e "$PROJ/spec.toml"` 不成立（spec.toml 不属部署载荷）
- [ ] REQ-014 一致性：`sqlite3 "$MOJIAN_HOME/central.db" "SELECT spec_version, spec_hash FROM project"` 两列均非空
- [ ] `target/debug/mojian status --path "$PROJ"` 退出码 0，stdout 含 `style_sampling`
- [ ] REQ-013 hash 覆盖：`echo tampered >> "$PROJ/CLAUDE.md"` 后 `target/debug/mojian status --path "$PROJ"` 退出码 0，随后 `grep -c tampered "$PROJ/CLAUDE.md"` 返回 `0`（被重部署覆盖还原）
- [ ] 桩命令：`target/debug/mojian run` 退出码 0 且 stdout 含 `stub，将在 ITER-002 实现`；`target/debug/mojian decide CH-001 CONFIRMED` 退出码 0 且 stdout 含同一提示
- [ ] 错误路径：`target/debug/mojian status --path "$(mktemp -d)"`（无 mojian.toml 的空目录）退出码非 0，stderr/stdout 含「非 mojian 项目」类错误信息
- [ ] 重复初始化：对已初始化的 `$PROJ` 再次 `target/debug/mojian new "$PROJ"` 退出码非 0（拒绝重复初始化）

## Dependencies

- 前置任务：TASK-004, TASK-005

## Log

- 2026-07-07 [planning-agent] status — → planned：创建任务
