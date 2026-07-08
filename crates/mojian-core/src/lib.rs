pub mod context;
pub mod db;
pub mod domain;
pub mod engine;
pub mod error;
pub mod log;
pub mod paths;
pub mod project;
pub mod sdk;
pub mod state;
pub mod spec;

pub use context::assemble_bundle;
pub use db::{open_central_db, SCHEMA_VERSION};
pub use domain::{ChapterState, ExtractStatus, SopPhase};
pub use engine::{
    apply_decision, apply_generation, load_run_state, next_action, Action, RunState, Verdict,
    BRIEF_GATE,
};
pub use error::CoreError;
pub use log::{
    append_decision, append_generation, read_decision_comments, DecisionEvent, GenerationEvent,
    InputSlice,
};
pub use state::{
    advance_sop_phase, clear_gate, insert_void_record, load_chapter, read_pending_gate, set_gate,
    update_chapter_status, upsert_artifact_ref, Chapter,
};
pub use project::{
    load_project_state, read_manifest, register_project, update_project_spec, write_manifest,
    ProjectManifest,
};
pub use sdk::{Bundle, ClaudeCliRunner, GenerationRunner, SdkResponse};
pub use spec::{
    authoritative_hash, authoritative_version, deploy_spec, embedded_spec, ensure_master,
    sync_if_drifted, tree_hash,
};
