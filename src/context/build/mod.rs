pub mod config;
pub mod folding;
pub mod fs_builder;

pub use config::ContextBuildConfig;
pub use fs_builder::build_context_index;

// Backward-compatible re-exports from the legacy fs_indexer module.
pub use crate::context::fs_indexer::{build_contexts_from_file_tree, FileTreeContextOptions};
