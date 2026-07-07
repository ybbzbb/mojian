//! SPEC 主副本管理：嵌入骨架落地、权威 version / hash、部署与 hash 漂移覆盖。

pub mod deploy;
pub mod hash;
pub mod master;

pub use deploy::{deploy_spec, sync_if_drifted};
pub use hash::tree_hash;
pub use master::{authoritative_hash, authoritative_version, embedded_spec, ensure_master};
