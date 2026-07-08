//! `mojian run [--path <dir>]`：创作循环。定位项目 → 打开时 hash 覆盖 → 循环
//! `engine::next_action`，按动作分支执行（对齐 tech-design.md「API 变更 / mojian run」）。
//!
//! CLI 层保持薄：状态机映射由 `mojian_core::engine::next_action` 决定，本处只负责
//! 装配 / 调 runner / 落日志 / 打印卡点。`Advance` 只在占位 no-op 前置 phase 之间纯推进。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use clap::Args;
use mojian_core::context::content_hash;
use mojian_core::domain::SopPhase;
use mojian_core::paths::{central_db_path, data_dir, spec_master_dir};
use mojian_core::{
    advance_sop_phase, apply_generation, assemble_bundle, authoritative_hash, authoritative_version,
    append_generation, ensure_master, load_run_state, next_action, open_central_db, read_manifest,
    read_decision_comments, sync_if_drifted, update_project_spec, Action, ClaudeCliRunner,
    GenerationEvent, GenerationRunner, InputSlice, BRIEF_GATE,
};

use crate::spec_assets::SPEC_ASSETS;

/// 循环上限：占位前置 phase 顺推有限，超过即视为异常，避免状态未变时空转。
const MAX_STEPS: usize = 32;

#[derive(Args)]
pub struct RunArgs {
    /// 目标项目目录（默认当前工作目录）。
    #[arg(long)]
    pub path: Option<PathBuf>,
}

pub fn run(args: RunArgs) -> Result<()> {
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

    // 前置：确保数据目录 / 主副本就位。
    let data = data_dir().context("解析客户端数据目录失败")?;
    std::fs::create_dir_all(&data)
        .with_context(|| format!("创建客户端数据目录失败：{}", data.display()))?;
    let master = spec_master_dir().context("解析 SPEC 主副本目录失败")?;
    ensure_master(&SPEC_ASSETS, &master).context("落地 SPEC 主副本失败")?;

    let db_path = central_db_path().context("解析中央 DB 路径失败")?;
    let conn = open_central_db(&db_path).context("打开中央 DB 失败")?;

    // 打开时 hash 覆盖：漂移则重部署并回填 DB spec 列（与 status 同口径）。
    let auth_hash = authoritative_hash(&master).context("计算权威 SPEC hash 失败")?;
    let (overwritten, spec_hash) =
        sync_if_drifted(&project_dir, &auth_hash, &master).context("SPEC 同步失败")?;
    if overwritten {
        let version = authoritative_version(&master).context("读取权威 SPEC 版本失败")?;
        update_project_spec(&conn, &project_id, &version, &spec_hash)
            .context("回填 project 行 spec 列失败")?;
    }

    let runner = ClaudeCliRunner::new(&project_dir);

    for _ in 0..MAX_STEPS {
        let state = load_run_state(&conn, &project_id)
            .with_context(|| format!("读取运行状态失败（project_id={project_id}）"))?;
        match next_action(&state) {
            Action::Advance => {
                let next = next_placeholder_phase(state.sop_phase).with_context(|| {
                    format!("无法推进占位 phase：{}", state.sop_phase.as_db_str())
                })?;
                advance_sop_phase(&conn, &project_id, next).context("推进 SOP phase 失败")?;
            }
            Action::Generate { agent, manifest } => {
                let manifest_path = project_dir.join(&manifest);
                let bundle = assemble_bundle(&conn, &project_id, &project_dir, &manifest_path)
                    .context("装配输入 bundle 失败")?;
                let response = runner.run(&bundle).context("调用生成命令失败")?;

                // generation.jsonl 输入溯源：把本步回喂的人类关卡评论记为输入切片（REQ-011），
                // 使评论文本随生成事件落盘、供后续过期检测 / token 对账。
                let comments = read_decision_comments(&project_id, BRIEF_GATE, None)
                    .context("读回关卡评论失败")?;
                let inputs: Vec<InputSlice> = comments
                    .iter()
                    .map(|comment| InputSlice {
                        path: "decision.jsonl".to_string(),
                        anchor: Some(comment.clone()),
                        content_hash: content_hash(comment.as_bytes()),
                    })
                    .collect();

                let event = GenerationEvent {
                    step: state.sop_phase.as_db_str().to_string(),
                    agent: agent.clone(),
                    spec_path: manifest.clone(),
                    spec_hash: spec_hash.clone(),
                    inputs: inputs.clone(),
                    token_in: response.usage_in,
                    token_out: response.usage_out,
                    cost: response.cost,
                    ts: now_iso8601(),
                };
                append_generation(&project_id, &event).context("写 generation.jsonl 失败")?;
                apply_generation(&conn, &project_id, BRIEF_GATE, &inputs)
                    .context("生成落库置关卡失败")?;

                println!("已完成生成步：{agent}");
                println!(
                    "卡在 {BRIEF_GATE} 关卡，等待判定：mojian decide {BRIEF_GATE} CONFIRMED|REVISE|VOID"
                );
                return Ok(());
            }
            Action::HumanGate { gate } => {
                println!("卡在 {gate} 关卡，等待判定：mojian decide {gate} CONFIRMED|REVISE|VOID");
                return Ok(());
            }
            Action::Idle => {
                println!(
                    "无待执行动作（当前 phase: {}），正常退出。",
                    state.sop_phase.as_db_str()
                );
                return Ok(());
            }
        }
    }

    bail!("run 循环超过 {MAX_STEPS} 步仍未到达关卡 / 出口，疑似状态未推进")
}

/// 占位前置 phase 的顺推后继（对齐 tech-design.md「占位简化」：仅 `style_sampling` /
/// `style_extracting` 为 no-op 直进）。`next_action` 只对这两个 phase 出 `Advance`，
/// 故此处只需覆盖它们；其余 phase 不应到达本函数。
fn next_placeholder_phase(phase: SopPhase) -> Option<SopPhase> {
    match phase {
        SopPhase::StyleSampling => Some(SopPhase::StyleExtracting),
        SopPhase::StyleExtracting => Some(SopPhase::BriefDrafting),
        _ => None,
    }
}

/// 生成 UTC RFC3339 时间戳（秒级）。CLI 未引入 `time` 依赖，用 std 由 UNIX 秒做
/// 民用日期换算（Howard Hinnant days-from-civil 逆算法）。
pub(crate) fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year_civil = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year_civil + 1 } else { year_civil };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
