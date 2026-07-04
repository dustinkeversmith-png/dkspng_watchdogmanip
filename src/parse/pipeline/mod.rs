use crate::parse::boundary::BoundarySolver;
use crate::parse::hierarchy::resolve_hierarchy;
use crate::parse::model::{ParseOutput, SourceDocument};
use crate::parse::passes::{
    extract::extract_block_with_registry, infer::infer_loose_objects, normalize::normalize_command,
    validate::validate_commands,
};
use crate::parse::registry::CommandRegistry;
use crate::parse::seeds::CommandSeedDetector;
use crate::parse::shape::CommandShapeDetector;

pub mod context;

pub use context::ParseContext;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_loose_inference: bool,
    pub preserve_unknown_commands: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_loose_inference: true,
            preserve_unknown_commands: true,
        }
    }
}

pub struct MacroPipeline {
    command_registry: CommandRegistry,
    command_seed_detector: CommandSeedDetector,
    boundary_solver: BoundarySolver,
    config: PipelineConfig,
}

impl Default for MacroPipeline {
    fn default() -> Self {
        Self::with_defaults(PipelineConfig::default())
    }
}

impl MacroPipeline {
    pub fn with_defaults(config: PipelineConfig) -> Self {
        Self {
            command_registry: CommandRegistry::default(),
            command_seed_detector: CommandSeedDetector::with_defaults(),
            boundary_solver: BoundarySolver::with_defaults(),
            config,
        }
    }

    pub fn new(
        command_registry: CommandRegistry,
        command_seed_detector: CommandSeedDetector,
        boundary_solver: BoundarySolver,
        config: PipelineConfig,
    ) -> Self {
        Self {
            command_registry,
            command_seed_detector,
            boundary_solver,
            config,
        }
    }

    pub fn command_registry(&self) -> &CommandRegistry {
        &self.command_registry
    }

    pub fn command_seed_detector(&self) -> &CommandSeedDetector {
        &self.command_seed_detector
    }

    pub fn boundary_solver(&self) -> &BoundarySolver {
        &self.boundary_solver
    }

    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    pub fn with_command_registry(mut self, command_registry: CommandRegistry) -> Self {
        self.command_registry = command_registry;
        self
    }

    pub fn with_command_seed_detector(
        mut self,
        command_seed_detector: CommandSeedDetector,
    ) -> Self {
        self.command_seed_detector = command_seed_detector;
        self
    }

    pub fn with_boundary_solver(mut self, boundary_solver: BoundarySolver) -> Self {
        self.boundary_solver = boundary_solver;
        self
    }

    pub fn parse(&self, source_name: impl Into<String>, input: impl Into<String>) -> ParseOutput {
        let doc = SourceDocument::new(source_name, input);
        self.parse_document(doc)
    }

    pub fn parse_document(&self, doc: SourceDocument) -> ParseOutput {
        let ctx = ParseContext::new(&doc, &self.command_registry);
        let mut seeds = self.command_seed_detector.detect(&ctx);
        if self.config.enable_loose_inference {
            seeds.extend(infer_loose_objects(&doc, &seeds));
            seeds.sort_by_key(|s| s.start_line_index);
        }
        if !self.config.preserve_unknown_commands {
            seeds.retain(|s| {
                !matches!(
                    s.canonical_kind,
                    crate::parse::model::CommandKind::Unknown(_)
                )
            });
        }
        let mut blocks = self.boundary_solver.assemble_blocks(&ctx, &seeds);
        for (idx, block) in blocks.iter_mut().enumerate() {
            block.shape_analysis = Some(CommandShapeDetector::analyze(block, idx));
        }
        let mut commands: Vec<_> = blocks
            .iter()
            .enumerate()
            .map(|(i, b)| extract_block_with_registry(b, i, Some(&self.command_registry)))
            .collect();
        for cmd in &mut commands {
            normalize_command(cmd);
        }
        let diagnostics = validate_commands(&commands);
        let hierarchy = resolve_hierarchy(&doc, &blocks, &mut commands);
        ParseOutput {
            source_name: doc.source_name,
            commands,
            diagnostics,
            hierarchy,
        }
    }
}
