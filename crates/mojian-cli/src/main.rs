//! `mojian` 二进制入口：clap v4 derive 解析命令面并分发到各子命令。
//!
//! 命令契约见迭代 tech-design.md「API 变更」：`new` 建项目、`status` 读 SOP phase，
//! `run` 推进生成循环至人工关卡、`decide` 在关卡提交决定。子命令实现返回 `anyhow::Result`，顶层据此决定退出码。

mod commands;
mod spec_assets;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mojian", version, about = "墨简：小说创作 SOP 命令行工具")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 新建项目：建目录 + 登记中央 DB + 部署 SPEC + 写 mojian.toml
    New(commands::new::NewArgs),
    /// 查看项目当前 SOP phase（打开时按 hash 同步 SPEC）
    Status(commands::status::StatusArgs),
    /// 推进生成循环：装配上下文 → 调 SDK 生成 → 撞人工关卡即停
    Run(commands::run::RunArgs),
    /// 在人工关卡提交决定：CONFIRMED / REVISE / VOID（可带评论）
    Decide(commands::decide::DecideArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New(args) => commands::new::run(args),
        Command::Status(args) => commands::status::run(args),
        Command::Run(args) => commands::run::run(args),
        Command::Decide(args) => commands::decide::run(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("错误：{err:#}");
            ExitCode::FAILURE
        }
    }
}
