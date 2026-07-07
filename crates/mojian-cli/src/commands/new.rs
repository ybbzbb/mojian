//! `mojian new <dir>`：建项目 + 中央 DB 登记 + SPEC 部署 + 写 manifest。
//!
//! 有序 6 步见 tech-design.md「API 变更 / mojian new」：校验 dir → 前置就绪
//! （数据目录 / 建库 / 主副本 bootstrap）→ 事务登记 project+project_state（初始
//! `style_sampling`）→ 部署 SPEC 得 version/hash → 回填 project 行 → 写 mojian.toml。

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Args;
use mojian_core::paths::{central_db_path, data_dir, spec_master_dir};
use mojian_core::{
    deploy_spec, ensure_master, open_central_db, register_project, update_project_spec,
    write_manifest, ProjectManifest,
};

use crate::spec_assets::SPEC_ASSETS;

#[derive(Args)]
pub struct NewArgs {
    /// 项目目录路径（相对 / 绝对均可，内部转绝对路径）。
    pub dir: PathBuf,
}

pub fn run(args: NewArgs) -> Result<()> {
    // 1. 校验 <dir>：已含 mojian.toml → 拒绝重复初始化；否则确保目录存在。
    if args.dir.join("mojian.toml").exists() {
        bail!(
            "目录已是 mojian 项目（存在 mojian.toml），拒绝重复初始化：{}",
            args.dir.display()
        );
    }
    std::fs::create_dir_all(&args.dir)
        .with_context(|| format!("创建项目目录失败：{}", args.dir.display()))?;
    let project_dir = absolutize(&args.dir).context("解析项目目录绝对路径失败")?;

    // 前置：确保数据目录 / 建库 / 主副本 bootstrap（首次运行落地嵌入骨架）。
    let data = data_dir().context("解析客户端数据目录失败")?;
    std::fs::create_dir_all(&data)
        .with_context(|| format!("创建客户端数据目录失败：{}", data.display()))?;
    let master = spec_master_dir().context("解析 SPEC 主副本目录失败")?;
    ensure_master(&SPEC_ASSETS, &master).context("落地 SPEC 主副本失败")?;

    let db_path = central_db_path().context("解析中央 DB 路径失败")?;
    let mut conn = open_central_db(&db_path).context("打开中央 DB 失败")?;

    // 3. 事务内登记 project + project_state（初始 style_sampling）。
    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("mojian-project")
        .to_string();
    let project_id =
        register_project(&mut conn, &name, &project_dir).context("登记项目到中央 DB 失败")?;

    // 4. 部署 SPEC 主副本 → 项目目录，得权威 version / hash。
    let (spec_version, spec_hash) =
        deploy_spec(&master, &project_dir).context("部署 SPEC 到项目目录失败")?;

    // 5. 回填 project 行的 spec_version / spec_hash（REQ-014）。
    update_project_spec(&conn, &project_id, &spec_version, &spec_hash)
        .context("回填 project 行 spec 列失败")?;

    // 6. 写项目身份标记 mojian.toml。
    let manifest = ProjectManifest {
        project_id: project_id.clone(),
        spec_version: spec_version.clone(),
    };
    write_manifest(&project_dir, &manifest).context("写 mojian.toml 失败")?;

    println!("project_id: {project_id}");
    println!("path: {}", project_dir.display());
    println!("phase: style_sampling");
    Ok(())
}

/// 规整为绝对路径（不解析符号链接，保持与 core 存库口径一致，避免 macOS `/var`→`/private/var` 漂移）。
fn absolutize(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cwd = std::env::current_dir().context("读取当前工作目录失败")?;
        Ok(cwd.join(path))
    }
}
