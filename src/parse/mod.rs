//! Inline Macro Processor
//!
//! A modular, ambiguity-tolerant parser for notes that mix explicit `@commands`,
//! markdown-ish structure, inline statuses, references, paths, loose prose, and
//! inferred objectives. The crate is organized as independent passes so an agent
//! can add behaviors without coupling extraction logic to command definitions.

pub mod model;
pub mod passes;
pub mod pipeline;
pub mod database;
pub mod registry;

pub use self::model::*;
pub use self::pipeline::{MacroPipeline, PipelineConfig};
pub use self::registry::{default_registry, CommandRegistry};

pub use self::database::*;
