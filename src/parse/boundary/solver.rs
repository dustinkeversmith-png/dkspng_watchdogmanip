use crate::parse::boundary::model::{BoundaryCandidate, BoundaryMarkerKind, CommandBlock, ParseDocumentInput};
use crate::parse::boundary::strategies::BoundaryStrategyRegistry;
use crate::parse::model::{BoundaryKind, CommandSeed};
use crate::parse::pipeline::ParseContext;

pub trait BlockAssemblyStrategy {
    fn name(&self) -> &'static str;
    fn assemble_blocks(
        &self,
        ctx: &ParseContext,
        seeds: &[CommandSeed],
    ) -> Vec<CommandBlock>;
}

#[derive(Default)]
pub struct BlockAssemblerRegistry {
    assemblers: Vec<Box<dyn BlockAssemblyStrategy>>,
}

impl BlockAssemblerRegistry {
    pub fn new() -> Self {
        Self {
            assemblers: Vec::new(),
        }
    }

    pub fn register(&mut self, assembler: Box<dyn BlockAssemblyStrategy>) {
        self.assemblers.push(assembler);
    }

    pub fn assemble_blocks(&self, ctx: &ParseContext, seeds: &[CommandSeed]) -> Vec<CommandBlock> {
        for assembler in &self.assemblers {
            let blocks = assembler.assemble_blocks(ctx, seeds);
            if !blocks.is_empty() || seeds.is_empty() {
                return blocks;
            }
        }
        Vec::new()
    }
}

impl BlockAssemblerRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(NextSeedBlockAssemblyStrategy));
        registry
    }
}

#[derive(Default)]
pub struct BoundarySolver {
    boundary_strategies: BoundaryStrategyRegistry,
    block_assemblers: BlockAssemblerRegistry,
}

impl BoundarySolver {
    pub fn new(
        boundary_strategies: BoundaryStrategyRegistry,
        block_assemblers: BlockAssemblerRegistry,
    ) -> Self {
        Self {
            boundary_strategies,
            block_assemblers,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(
            BoundaryStrategyRegistry::with_defaults(),
            BlockAssemblerRegistry::with_defaults(),
        )
    }

    pub fn boundary_strategies(&self) -> &BoundaryStrategyRegistry {
        &self.boundary_strategies
    }

    pub fn boundary_strategies_mut(&mut self) -> &mut BoundaryStrategyRegistry {
        &mut self.boundary_strategies
    }

    pub fn block_assemblers(&self) -> &BlockAssemblerRegistry {
        &self.block_assemblers
    }

    pub fn block_assemblers_mut(&mut self) -> &mut BlockAssemblerRegistry {
        &mut self.block_assemblers
    }

    pub fn collect_boundary_candidates(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        self.boundary_strategies.collect_candidates(document)
    }

    pub fn assemble_blocks(&self, ctx: &ParseContext, seeds: &[CommandSeed]) -> Vec<CommandBlock> {
        self.block_assemblers.assemble_blocks(ctx, seeds)
    }
}

pub struct NextSeedBlockAssemblyStrategy;

impl BlockAssemblyStrategy for NextSeedBlockAssemblyStrategy {
    fn name(&self) -> &'static str {
        "next_seed"
    }

    fn assemble_blocks(
        &self,
        ctx: &ParseContext,
        seeds: &[CommandSeed],
    ) -> Vec<CommandBlock> {
        let doc = ctx.document;
        let mut blocks = Vec::new();
        for (idx, seed) in seeds.iter().enumerate() {
            let start_line = seed.start_line_index;
            let next_start = seeds
                .get(idx + 1)
                .map(|s| s.start_line_index)
                .unwrap_or(doc.lines.len());
            let seed_indent = doc.lines[start_line].indent;
            let mut end_line_exclusive = next_start;
            let mut boundary = BoundaryKind::UntilNextCommand;

            for li in (start_line + 1)..next_start {
                let line = &doc.lines[li];
                let trimmed = line.text.trim();
                if trimmed.is_empty() {
                    let next_nonblank =
                        ((li + 1)..next_start).find(|n| !doc.lines[*n].text.trim().is_empty());
                    if let Some(n) = next_nonblank {
                        if doc.lines[n].indent <= seed_indent
                            && doc.lines[n].text.trim_start().starts_with('#')
                        {
                            end_line_exclusive = li;
                            boundary = BoundaryKind::UntilBlankLine;
                            break;
                        }
                    }
                }
                if li > start_line + 1 && line.indent < seed_indent && !trimmed.starts_with('-') {
                    end_line_exclusive = li;
                    boundary = BoundaryKind::UntilOutdent;
                    break;
                }
            }

            if end_line_exclusive == doc.lines.len() {
                boundary = BoundaryKind::EndOfDocument;
            }
            let mut body_lines = Vec::new();
            if !seed.inline_payload.is_empty() {
                body_lines.push(seed.inline_payload.clone());
            }
            for li in (start_line + 1)..end_line_exclusive {
                body_lines.push(doc.lines[li].text.clone());
            }
            let end_idx = end_line_exclusive.saturating_sub(1).max(start_line);
            let end_char = doc
                .lines
                .get(end_idx)
                .map(|l| l.end)
                .unwrap_or(seed.span.end);
            let end_line_no = doc
                .lines
                .get(end_idx)
                .map(|l| l.number)
                .unwrap_or(seed.span.line_end);
            blocks.push(CommandBlock {
                seed: seed.clone(),
                body_lines,
                span: crate::parse::model::TextSpan::new(
                    seed.span.start,
                    end_char,
                    seed.span.line_start,
                    end_line_no,
                ),
                boundary_kind: boundary,
                location: crate::parse::model::SourceLocation::from_span(
                    doc.source_name.clone(),
                    doc.file_path.clone(),
                    crate::parse::model::TextSpan::new(
                        seed.span.start,
                        end_char,
                        seed.span.line_start,
                        end_line_no,
                    ),
                ),
                shape_analysis: None,
            });
        }
        blocks
    }
}

pub fn map_legacy_boundary(
    seed: &CommandSeed,
    next_seed_line: Option<usize>,
    doc_len: usize,
) -> BoundaryMarkerKind {
    if next_seed_line.is_none() && seed.start_line_index + 1 >= doc_len {
        BoundaryMarkerKind::CommandEnd
    } else if next_seed_line.is_some() {
        BoundaryMarkerKind::NextSeedBoundary
    } else {
        BoundaryMarkerKind::BlockCommand
    }
}
