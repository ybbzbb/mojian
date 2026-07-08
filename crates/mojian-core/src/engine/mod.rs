//! 状态机核心：纯函数 `next_action` + `apply_*` 落库收敛。
//!
//! 守 AP-001「状态机是代码不是散文」：`next_action` 是 `RunState -> Action` 的纯映射，
//! 不触 IO；`apply_generation` / `apply_decision` 把生成 / 判定的结果收敛到 DB。
//!
//! 本迭代占位简化（tech-design.md「依赖与风险 · 占位简化」）：SOP① `style_sampling` /
//! `style_extracting` 为 no-op 直进（`Advance`），仅 `brief_drafting` 为完整 `Generate`
//! 步、`brief` 为完整关卡；更深 phase 未接线时诚实出 `Idle`。`Generate` 与关卡之间
//! **预留客观检查步位**（裁决①），本迭代空过（不写 check.jsonl）。

use rusqlite::Connection;

use crate::domain::{ChapterState, SopPhase};
use crate::error::CoreError;
use crate::log::InputSlice;
use crate::state;

/// SOP① `brief_drafting` 步对应的人工关卡名。
pub const BRIEF_GATE: &str = "brief";

/// `brief_drafting` 步的部署 agent 提示词相对路径。
const BRIEF_AGENT: &str = ".claude/agents/brief-agent.md";

/// `brief_drafting` 步的输入契约 manifest sidecar 相对路径。
const BRIEF_MANIFEST: &str = ".claude/agents/brief-agent.manifest.toml";

/// `next_action` 计算出的下一个动作（对齐 tech-design.md「API 变更 / mojian run」分支）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 纯推进 `sop_phase`（占位 no-op 直进）。
    Advance,
    /// 执行一次 SDK 生成步：装配 `manifest` → 调 runner → 落库置关卡。
    Generate { agent: String, manifest: String },
    /// 撞上人工关卡，停机等判定。
    HumanGate { gate: String },
    /// 无可跑动作（深层 phase 未接线的诚实出口）。
    Idle,
}

/// 人在关卡的判定（对齐 REQ-009）：CONFIRMED 通过、REVISE 回喂、VOID 作废章节。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Confirmed,
    Revise,
    Void,
}

impl Verdict {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Verdict::Confirmed => "CONFIRMED",
            Verdict::Revise => "REVISE",
            Verdict::Void => "VOID",
        }
    }
}

impl TryFrom<&str> for Verdict {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let verdict = match value {
            "CONFIRMED" => Verdict::Confirmed,
            "REVISE" => Verdict::Revise,
            "VOID" => Verdict::Void,
            _ => {
                return Err(CoreError::UnknownDomainValue {
                    kind: "Verdict",
                    value: value.to_string(),
                })
            }
        };
        Ok(verdict)
    }
}

/// `next_action` 的纯输入：当前 SOP phase + 是否卡在关卡。由 `load_run_state` 从
/// `project_state` 读出，也可在单元测中直接构造（纯函数、无 IO）。
#[derive(Debug, Clone)]
pub struct RunState {
    pub sop_phase: SopPhase,
    pub pending_gate: Option<String>,
}

/// 从 `project_state` 读出当前 `RunState`（供 CLI `run` 循环喂给 `next_action`）。
pub fn load_run_state(conn: &Connection, project_id: &str) -> Result<RunState, CoreError> {
    let sop_phase = crate::project::load_project_state(conn, project_id)?;
    let pending_gate = state::read_pending_gate(conn, project_id)?;
    Ok(RunState {
        sop_phase,
        pending_gate,
    })
}

/// **纯函数**：由 `RunState` 计算下一个动作。
///
/// 关卡优先——`pending_gate` 存在即出 `HumanGate`（`Generate` 与关卡之间预留的客观检查步位
/// 本迭代空过）。否则按 phase 映射：`style_sampling` / `style_extracting` 占位直进
/// `Advance`；`brief_drafting` 出 `Generate`；其余未接线 phase 出 `Idle`。
pub fn next_action(state: &RunState) -> Action {
    if let Some(gate) = &state.pending_gate {
        return Action::HumanGate { gate: gate.clone() };
    }

    match state.sop_phase {
        SopPhase::StyleSampling | SopPhase::StyleExtracting => Action::Advance,
        SopPhase::BriefDrafting => Action::Generate {
            agent: BRIEF_AGENT.to_string(),
            manifest: BRIEF_MANIFEST.to_string(),
        },
        _ => Action::Idle,
    }
}

/// 生成步落库收敛：置 `gate` 关卡标记（`cursors.pending_gate`）+ 为每个输入切片
/// upsert `artifact_ref`（`content_hash` 与 generation.jsonl 同源）。
///
/// generation.jsonl 的写入由调用方（`run` 循环）在调 SDK 后完成；本函数只收敛 DB 面。
pub fn apply_generation(
    conn: &Connection,
    project_id: &str,
    gate: &str,
    inputs: &[InputSlice],
) -> Result<(), CoreError> {
    for slice in inputs {
        state::upsert_artifact_ref(conn, project_id, &slice.path, "input", &slice.content_hash)?;
    }
    state::set_gate(conn, project_id, gate)?;
    Ok(())
}

/// 判定落库收敛（对齐 tech-design.md「API 变更 · decide 行为」）：
///
/// - `CONFIRMED`：清关卡 + 推进（`brief` → `vision_drafting`）。
/// - `REVISE`：清关卡 + 回退对应细粒度状态（`brief` 关卡回 `brief_drafting`；章节关卡回
///   `skeleton_drafting`）；评论留待下次装配回喂（REQ-011，本函数不动 decision.jsonl）。
/// - `VOID <CH>`：写 `void_record` + `chapter.status: void → planned`（裁决③ 最小语义，
///   不做圣经级联 / 过期检测）。
pub fn apply_decision(
    conn: &Connection,
    project_id: &str,
    gate: &str,
    verdict: Verdict,
    target: Option<&str>,
) -> Result<(), CoreError> {
    match verdict {
        Verdict::Confirmed => {
            state::clear_gate(conn, project_id)?;
            if gate == BRIEF_GATE {
                state::advance_sop_phase(conn, project_id, SopPhase::VisionDrafting)?;
            }
        }
        Verdict::Revise => {
            state::clear_gate(conn, project_id)?;
            if gate == BRIEF_GATE {
                state::advance_sop_phase(conn, project_id, SopPhase::BriefDrafting)?;
            } else if let Some(chapter_id) = target {
                state::update_chapter_status(conn, chapter_id, ChapterState::SkeletonDrafting)?;
            }
        }
        Verdict::Void => {
            let chapter_id = target.ok_or_else(|| CoreError::MissingDecisionTarget {
                verdict: Verdict::Void.as_db_str().to_string(),
            })?;
            state::insert_void_record(conn, project_id, chapter_id, None, None)?;
            state::update_chapter_status(conn, chapter_id, ChapterState::Planned)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_state(phase: SopPhase, gate: Option<&str>) -> RunState {
        RunState {
            sop_phase: phase,
            pending_gate: gate.map(str::to_string),
        }
    }

    #[test]
    fn style_phases_map_to_advance() {
        assert_eq!(next_action(&run_state(SopPhase::StyleSampling, None)), Action::Advance);
        assert_eq!(next_action(&run_state(SopPhase::StyleExtracting, None)), Action::Advance);
    }

    #[test]
    fn brief_drafting_maps_to_generate() {
        let action = next_action(&run_state(SopPhase::BriefDrafting, None));
        assert_eq!(
            action,
            Action::Generate {
                agent: BRIEF_AGENT.to_string(),
                manifest: BRIEF_MANIFEST.to_string(),
            }
        );
    }

    #[test]
    fn pending_gate_takes_priority_over_phase() {
        // brief_drafting 本会出 Generate，但一旦置关卡，必须优先出 HumanGate。
        let action = next_action(&run_state(SopPhase::BriefDrafting, Some("brief")));
        assert_eq!(action, Action::HumanGate { gate: "brief".to_string() });
    }

    #[test]
    fn unwired_phase_maps_to_idle() {
        assert_eq!(next_action(&run_state(SopPhase::VisionDrafting, None)), Action::Idle);
        assert_eq!(next_action(&run_state(SopPhase::Writing, None)), Action::Idle);
    }

    #[test]
    fn verdict_parses_and_round_trips() {
        assert_eq!(Verdict::try_from("CONFIRMED").unwrap(), Verdict::Confirmed);
        assert_eq!(Verdict::try_from("REVISE").unwrap(), Verdict::Revise);
        assert_eq!(Verdict::try_from("VOID").unwrap(), Verdict::Void);
        assert_eq!(Verdict::Confirmed.as_db_str(), "CONFIRMED");
        assert!(matches!(
            Verdict::try_from("nope").unwrap_err(),
            CoreError::UnknownDomainValue { kind: "Verdict", .. }
        ));
    }
}
