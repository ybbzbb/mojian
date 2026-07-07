//! `mojian status [--path <dir>]`：读 manifest + 打开时 hash 覆盖 + 输出 SOP phase。
//!
//! 有序 3 步见 tech-design.md「API 变更 / mojian status」：读目标目录 mojian.toml 取
//! project_id（缺失 → 报「非 mojian 项目」非 0 退出）→ `sync_if_drifted` 打开时 hash
//! 覆盖（不一致则重部署并回填 DB spec 列）→ 按 project_id 读 sop_phase 并打印。

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;
use mojian_core::paths::{central_db_path, data_dir, spec_master_dir};
use mojian_core::{
    authoritative_hash, authoritative_version, ensure_master, load_project_state, open_central_db,
    read_manifest, sync_if_drifted, update_project_spec,
};

use crate::spec_assets::SPEC_ASSETS;

#[derive(Args)]
pub struct StatusArgs {
    /// 目标项目目录（默认当前工作目录）。
    #[arg(long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let project_dir = match args.path {
        Some(p) => p,
        None => std::env::current_dir().context("读取当前工作目录失败")?,
    };

    // 1. 读 manifest 取 project_id；缺 mojian.toml → 非 mojian 项目。
    if !project_dir.join("mojian.toml").exists() {
        bail!(
            "非 mojian 项目：目录下无 mojian.toml（{}）",
            project_dir.display()
        );
    }
    let manifest = read_manifest(&project_dir)
        .with_context(|| format!("读取 mojian.toml 失败：{}", project_dir.display()))?;
    let project_id = manifest.project_id;

    // 前置：确保主副本就位（首次运行或缺失时落地嵌入骨架）。
    let data = data_dir().context("解析客户端数据目录失败")?;
    std::fs::create_dir_all(&data)
        .with_context(|| format!("创建客户端数据目录失败：{}", data.display()))?;
    let master = spec_master_dir().context("解析 SPEC 主副本目录失败")?;
    ensure_master(&SPEC_ASSETS, &master).context("落地 SPEC 主副本失败")?;

    let db_path = central_db_path().context("解析中央 DB 路径失败")?;
    let conn = open_central_db(&db_path).context("打开中央 DB 失败")?;

    // 2. 打开时 hash 覆盖：漂移则重部署并回填 DB spec 列。
    let auth_hash = authoritative_hash(&master).context("计算权威 SPEC hash 失败")?;
    let (overwritten, new_hash) =
        sync_if_drifted(&project_dir, &auth_hash, &master).context("SPEC 同步失败")?;
    if overwritten {
        let version = authoritative_version(&master).context("读取权威 SPEC 版本失败")?;
        update_project_spec(&conn, &project_id, &version, &new_hash)
            .context("回填 project 行 spec 列失败")?;
    }

    // 3. 读回 SOP phase 并打印项目名 + 当前 phase。
    let phase = load_project_state(&conn, &project_id)
        .with_context(|| format!("读取项目状态失败（project_id={project_id}）"))?;
    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mojian-project");
    println!("project: {name}");
    println!("phase: {}", phase.as_db_str());
    Ok(())
}
