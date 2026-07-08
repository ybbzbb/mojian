//! `mojian decide <gate> <verdict> [target] [--comment "..." | --file <path>]`：提交人工决策。
//!
//! 对齐 tech-design.md「API 变更 / mojian decide」：校验当前确在 `<gate>`（否则非 0 退出，
//! 关卡状态不匹配错误）→ `log::append_decision` → `engine::apply_decision`（CONFIRMED 推进 /
//! REVISE 回退 + 评论回喂 / VOID 章节最小语义）。CLI 层薄：解析 → 校验 → 调 core → 打印。

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;
use mojian_core::paths::{central_db_path, data_dir};
use mojian_core::{
    apply_decision, append_decision, open_central_db, read_manifest, read_pending_gate,
    CoreError, DecisionEvent, Verdict,
};

use crate::commands::run::now_iso8601;

#[derive(Args)]
pub struct DecideArgs {
    /// 关卡名（如 `brief`）。
    pub gate: String,
    /// 判定：CONFIRMED | REVISE | VOID。
    pub verdict: String,
    /// 可选目标（章节 / 批 id，供 REVISE / VOID 使用）。
    pub target: Option<String>,
    /// 评论正文（REVISE 时留待下次装配回喂）。
    #[arg(long, conflicts_with = "file")]
    pub comment: Option<String>,
    /// 从文件读取评论内容（与 --comment 互斥）。
    #[arg(long)]
    pub file: Option<PathBuf>,
    /// 目标项目目录（默认当前工作目录）。
    #[arg(long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: DecideArgs) -> Result<()> {
    let project_dir = match args.path {
        Some(p) => p,
        None => std::env::current_dir().context("读取当前工作目录失败")?,
    };

    if !project_dir.join("mojian.toml").exists() {
        bail!(
            "非 mojian 项目：目录下无 mojian.toml（{}）",
            project_dir.display()
        );
    }
    let manifest = read_manifest(&project_dir)
        .with_context(|| format!("读取 mojian.toml 失败：{}", project_dir.display()))?;
    let project_id = manifest.project_id;

    let data = data_dir().context("解析客户端数据目录失败")?;
    std::fs::create_dir_all(&data)
        .with_context(|| format!("创建客户端数据目录失败：{}", data.display()))?;
    let db_path = central_db_path().context("解析中央 DB 路径失败")?;
    let conn = open_central_db(&db_path).context("打开中央 DB 失败")?;

    let verdict = Verdict::try_from(args.verdict.as_str())
        .with_context(|| format!("无效判定值：{}（须为 CONFIRMED|REVISE|VOID）", args.verdict))?;

    // 校验当前确在 <gate>：pending_gate 必须与请求 gate 一致，否则关卡状态不匹配、非 0 退出。
    let pending = read_pending_gate(&conn, &project_id)
        .with_context(|| format!("读取当前关卡失败（project_id={project_id}）"))?;
    match pending.as_deref() {
        Some(g) if g == args.gate => {}
        other => {
            return Err(CoreError::GateStateMismatch {
                expected: args.gate.clone(),
                actual: other.unwrap_or("<无关卡>").to_string(),
            }
            .into());
        }
    }

    let comment = resolve_comment(args.comment, args.file)?;

    let event = DecisionEvent {
        gate: args.gate.clone(),
        verdict: verdict.as_db_str().to_string(),
        target: args.target.clone(),
        comment: comment.clone(),
        ts: now_iso8601(),
    };
    append_decision(&project_id, &event).context("写 decision.jsonl 失败")?;
    apply_decision(&conn, &project_id, &args.gate, verdict, args.target.as_deref())
        .context("判定落库失败")?;

    println!("已记录判定：{} {}", args.gate, verdict.as_db_str());
    if let Some(target) = &args.target {
        println!("目标：{target}");
    }
    Ok(())
}

/// 解析评论来源：`--file` 优先读文件内容，否则用 `--comment`；均无则无评论。
fn resolve_comment(comment: Option<String>, file: Option<PathBuf>) -> Result<Option<String>> {
    if let Some(path) = file {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("读取评论文件失败：{}", path.display()))?;
        return Ok(Some(text.trim().to_string()));
    }
    Ok(comment)
}
