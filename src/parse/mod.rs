//! Inline Macro Processor
//!
//! A modular, ambiguity-tolerant parser for notes that mix explicit `@commands`,
//! markdown-ish structure, inline statuses, references, paths, loose prose, and
//! inferred objectives. The crate is organized as independent passes so an agent
//! can add behaviors without coupling extraction logic to command definitions.

pub mod boundary;
pub mod database;
pub mod extractors;
pub mod hierarchy;
pub mod model;
pub mod parser;
pub mod passes;
pub mod pipeline;
pub mod registry;
pub mod seeds;
pub mod shape;

pub use self::boundary::{
    BlockAssemblerRegistry, BlockAssemblyStrategy, BoundarySolver, BoundaryStrategy,
    BoundaryStrategyRegistry,
};
pub use self::model::*;
pub use self::parser::Parser;
pub use self::pipeline::{MacroPipeline, ParseContext, PipelineConfig};
pub use self::registry::{
    member, parameter, CommandBodyPolicy, CommandLayoutKind, CommandRegistry, CommandSpec,
    MemberSpec, ParameterSpec,
};
pub use self::seeds::{
    CommandSeedDetector, CommandSeedStrategy, CommandSeedStrategyRegistry, SeedDetectionStrategy,
    SeedDetectionStrategyRegistry, SeedDetector,
};

pub use self::database::*;
