use crate::parse::model::{ParseOutput, SourceDocument};
use crate::parse::passes::{
    boundary::solve_boundaries, detect::detect_command_seeds, extract::extract_block,
    infer::infer_loose_objects, normalize::normalize_command, validate::validate_commands,
};
use crate::parse::registry::{default_registry, CommandRegistry};

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

#[derive(Debug, Clone)]
pub struct MacroPipeline {
    registry: CommandRegistry,
    config: PipelineConfig,
}

impl Default for MacroPipeline {
    fn default() -> Self {
        Self::new(default_registry(), PipelineConfig::default())
    }
}

impl MacroPipeline {
    pub fn new(registry: CommandRegistry, config: PipelineConfig) -> Self {
        Self { registry, config }
    }

    pub fn parse(&self, source_name: impl Into<String>, input: impl Into<String>) -> ParseOutput {
        let doc = SourceDocument::new(source_name, input);
        let mut seeds = detect_command_seeds(&doc, &self.registry);
        if self.config.enable_loose_inference {
            seeds.extend(infer_loose_objects(&doc, &seeds));
            seeds.sort_by_key(|s| s.start_line_index);
        }
        if !self.config.preserve_unknown_commands {
            seeds.retain(|s| !matches!(s.canonical_kind, crate::parse::model::CommandKind::Unknown(_)));
        }
        let blocks = solve_boundaries(&doc, &seeds);
        let mut commands: Vec<_> = blocks
            .iter()
            .enumerate()
            .map(|(i, b)| extract_block(b, i))
            .collect();
        for cmd in &mut commands {
            normalize_command(cmd);
        }
        let diagnostics = validate_commands(&commands);
        ParseOutput {
            source_name: doc.source_name,
            commands,
            diagnostics,
        }
    }
}
