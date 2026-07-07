//! `mojian decide [args...]`：人工决策桩，将在 ITER-002 实现。
//!
//! 本迭代仅保证命令面存在、可被调用；接受并忽略尾随参数。

use anyhow::Result;
use clap::Args;

const STUB_MESSAGE: &str = "stub，将在 ITER-002 实现";

#[derive(Args)]
pub struct DecideArgs {
    /// 尾随参数（本迭代接受但忽略，ITER-002 落地语义）。
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub fn run(_args: DecideArgs) -> Result<()> {
    println!("{STUB_MESSAGE}");
    Ok(())
}
