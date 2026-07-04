use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::boundary::{BodyShapeHint, CommandBlock};
use crate::parse::shape::model::{
    CommandShapeAnalysis, CommandShapeKind, ParameterShapeKind, TitleCandidate, TitleCandidateKind,
};

static MEMBER_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?P<key>[A-Za-z][A-Za-z0-9 _/-]{1,40})\s*:\s*(?P<value>.*)$").unwrap()
});

pub struct CommandShapeDetector;

impl CommandShapeDetector {
    pub fn analyze(block: &CommandBlock, command_index: usize) -> CommandShapeAnalysis {
        let mut shape_kinds = Vec::new();
        let mut title_candidates = Vec::new();
        let mut diagnostics = Vec::new();

        let inline = block.seed.inline_payload.trim();
        if !inline.is_empty() {
            shape_kinds.push(CommandShapeKind::InlineTitle);
            shape_kinds.push(CommandShapeKind::InlineParameters);
            title_candidates.push(TitleCandidate {
                kind: TitleCandidateKind::InlineAfterCommand,
                text: inline.to_string(),
                line: block.seed.span.line_start,
                confidence: 0.85,
                reason: "inline payload after @command identity".to_string(),
            });
        }

        let parameter_shape = if inline.is_empty() {
            ParameterShapeKind::None
        } else if inline.split_whitespace().count() <= 2 {
            ParameterShapeKind::SingleLooseParameter
        } else {
            ParameterShapeKind::MultipleLooseParameters
        };

        let mut body_shape = BodyShapeHint::Empty;
        let body_text = block.body_lines.join("\n");
        let trimmed_body = body_text.trim();

        if trimmed_body.is_empty() && inline.is_empty() {
            shape_kinds.push(CommandShapeKind::EmptyBody);
            body_shape = BodyShapeHint::Empty;
        } else if trimmed_body.starts_with('[') || trimmed_body.contains('\n') && trimmed_body.contains('[') {
            shape_kinds.push(CommandShapeKind::BracketedBody);
            body_shape = BodyShapeHint::BracketedBlock;
        } else if block
            .body_lines
            .iter()
            .any(|l| MEMBER_LINE.is_match(l))
        {
            shape_kinds.push(CommandShapeKind::KeyValueMembers);
            body_shape = BodyShapeHint::KeyValueBlock;
            for (idx, line) in block.body_lines.iter().enumerate() {
                if let Some(caps) = MEMBER_LINE.captures(line) {
                    let key = caps.name("key").unwrap().as_str();
                    let value = caps.name("value").unwrap().as_str().trim();
                    if key.eq_ignore_ascii_case("title") {
                        title_candidates.push(TitleCandidate {
                            kind: TitleCandidateKind::TitleMember,
                            text: value.to_string(),
                            line: block.seed.span.line_start + idx + 1,
                            confidence: 0.92,
                            reason: "Title: member line".to_string(),
                        });
                    }
                }
            }
        } else if block.body_lines.iter().any(|l| l.starts_with("    ") || l.starts_with('\t')) {
            shape_kinds.push(CommandShapeKind::IndentedBody);
            body_shape = BodyShapeHint::IndentedBlock;
        } else if !trimmed_body.is_empty() {
            shape_kinds.push(CommandShapeKind::ProseOnly);
            body_shape = BodyShapeHint::FreeformProse;
            if let Some(first) = block.body_lines.iter().map(|l| l.trim()).find(|l| !l.is_empty()) {
                title_candidates.push(TitleCandidate {
                    kind: TitleCandidateKind::FirstNonEmptyBodyLine,
                    text: first.to_string(),
                    line: block.seed.span.line_start + 1,
                    confidence: 0.65,
                    reason: "first non-empty body line".to_string(),
                });
            }
        } else if !inline.is_empty() {
            body_shape = BodyShapeHint::SingleLine;
        }

        if shape_kinds.len() > 1 {
            shape_kinds.push(CommandShapeKind::Mixed);
            diagnostics.push("multiple shape kinds plausible".to_string());
        }
        if shape_kinds.is_empty() {
            shape_kinds.push(CommandShapeKind::Unknown);
            body_shape = BodyShapeHint::Unknown;
        }

        shape_kinds.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        shape_kinds.dedup();

        let confidence = title_candidates
            .iter()
            .map(|c| c.confidence)
            .fold(0.45_f32, f32::max);

        CommandShapeAnalysis {
            command_id: Some(format!("cmd_{:04}", command_index + 1)),
            shape_kinds,
            parameter_shape,
            body_shape,
            title_candidates,
            confidence,
            diagnostics,
        }
    }
}
