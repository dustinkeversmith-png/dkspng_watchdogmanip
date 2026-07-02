use crate::parse::model::{BoundaryKind, CommandSeed, SourceDocument, TextSpan};

#[derive(Debug, Clone)]
pub struct CommandBlock {
    pub seed: CommandSeed,
    pub body_lines: Vec<String>,
    pub span: TextSpan,
    pub boundary_kind: BoundaryKind,
}

pub fn solve_boundaries(doc: &SourceDocument, seeds: &[CommandSeed]) -> Vec<CommandBlock> {
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
                // blank line is soft: keep scanning if next nonblank is indented, otherwise close.
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
            span: TextSpan::new(seed.span.start, end_char, seed.span.line_start, end_line_no),
            boundary_kind: boundary,
        });
    }
    blocks
}
