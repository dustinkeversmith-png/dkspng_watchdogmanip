use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub line_start: usize,
    pub line_end: usize,
}

impl TextSpan {
    pub fn new(start: usize, end: usize, line_start: usize, line_end: usize) -> Self {
        Self {
            start,
            end,
            line_start,
            line_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLine {
    pub number: usize,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub indent: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDocument {
    pub source_name: String,
    pub text: String,
    pub lines: Vec<SourceLine>,
}

impl SourceDocument {
    pub fn new(source_name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut lines = Vec::new();
        let mut offset = 0usize;
        for (idx, raw) in text.split_inclusive('\n').enumerate() {
            let without_newline = raw.trim_end_matches(['\r', '\n']);
            let indent = without_newline
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count();
            let end = offset + raw.len();
            lines.push(SourceLine {
                number: idx + 1,
                start: offset,
                end,
                text: without_newline.to_string(),
                indent,
            });
            offset = end;
        }
        if text.is_empty() {
            lines.push(SourceLine {
                number: 1,
                start: 0,
                end: 0,
                text: String::new(),
                indent: 0,
            });
        } else if !text.ends_with('\n') {
            // split_inclusive already emitted the last unterminated line.
        }
        Self {
            source_name: source_name.into(),
            text,
            lines,
        }
    }
}
