# DevOps

## Config

vcs_platform: gh
review_policy: human_required

## Build Verification

Rust workspace（`cargo` stable）。`rusqlite` 的 `bundled` feature 会编译内置 SQLite，构建期需系统具备 C 编译器（macOS: Xcode CLT；Linux: cc/clang）。

- 检查：`cargo check`
- 构建：`cargo build --workspace`（REQ-002 验收口）
- 测试：`cargo test`（可配 `MOJIAN_HOME=<临时目录>` 指向隔离数据目录，避免污染真实 `~`）

运行产物：单二进制 `mojian`；运行期写 `<data_dir>/`（默认平台标准目录，`MOJIAN_HOME` 可覆盖），无外部服务依赖。
