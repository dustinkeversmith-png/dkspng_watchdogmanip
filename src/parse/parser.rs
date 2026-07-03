use crate::parse::model::{ParseOutput, SourceDocument};
use crate::parse::pipeline::{MacroPipeline, PipelineConfig};
use crate::parse::registry::{default_registry, CommandRegistry};

#[derive(Debug, Clone)]
pub struct Parser {
    inner: MacroPipeline,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(default_registry(), PipelineConfig::default())
    }
}

impl Parser {
    pub fn new(registry: CommandRegistry, config: PipelineConfig) -> Self {
        Self {
            inner: MacroPipeline::new(registry, config),
        }
    }

    pub fn parse(&self, source_name: impl Into<String>, text: impl Into<String>) -> ParseOutput {
        self.inner.parse(source_name, text)
    }

    pub fn parse_document(&self, doc: SourceDocument) -> ParseOutput {
        self.inner.parse(doc.source_name.clone(), doc.text)
    }
}
