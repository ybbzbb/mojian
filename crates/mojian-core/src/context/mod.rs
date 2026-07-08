//! 切片装配（IMPL-3）：TOML sidecar 输入契约 manifest → 符号引用解析 → 段级 / 整文件
//! 切片 + blake3 hash → 组装五字段 `Bundle`（含 decision.jsonl 人类评论回喂）。

pub mod assemble;
pub mod manifest;
pub mod slice;
pub mod symbol;

pub use assemble::assemble_bundle;
pub use manifest::{read_input_manifest, write_scope, InputManifest};
pub use slice::{content_hash, slice_ref, Slice};
pub use symbol::{resolve_symbol, SymbolRef};
