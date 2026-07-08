pub mod db;
pub mod domain;
pub mod error;
pub mod log;
pub mod paths;
pub mod project;
pub mod spec;

pub use db::{open_central_db, SCHEMA_VERSION};
pub use domain::{ChapterState, ExtractStatus, SopPhase};
pub use error::CoreError;
pub use log::{
    append_decision, append_generation, read_decision_comments, DecisionEvent, GenerationEvent,
    InputSlice,
};
pub use project::{
    load_project_state, read_manifest, register_project, update_project_spec, write_manifest,
    ProjectManifest,
};
pub use spec::{
    authoritative_hash, authoritative_version, deploy_spec, embedded_spec, ensure_master,
    sync_if_drifted, tree_hash,
};
