//! `mojian run`：创作循环桩，将在 ITER-002 实现。

use anyhow::Result;
use clap::Args;

const STUB_MESSAGE: &str = "stub，将在 ITER-002 实现";

#[derive(Args)]
pub struct RunArgs {}

pub fn run(_args: RunArgs) -> Result<()> {
    println!("{STUB_MESSAGE}");
    Ok(())
}
