use crate::parse::model::{ParseOutput, SourceDocument};
use crate::parse::pipeline::{MacroPipeline, PipelineConfig};
use crate::parse::registry::CommandRegistry;

pub struct Parser {
    inner: MacroPipeline,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            inner: MacroPipeline::default(),
        }
    }
}

impl Parser {
    pub fn new(registry: CommandRegistry, config: PipelineConfig) -> Self {
        Self {
            inner: MacroPipeline::with_defaults(config).with_command_registry(registry),
        }
    }

    pub fn parse(&self, source_name: impl Into<String>, text: impl Into<String>) -> ParseOutput {
        self.inner.parse(source_name, text)
    }

    pub fn parse_document(&self, doc: SourceDocument) -> ParseOutput {
        self.inner.parse_document(doc)
    }
}
