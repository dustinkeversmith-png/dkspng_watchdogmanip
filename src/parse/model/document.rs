use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub source_name: String,
    pub file_path: Option<PathBuf>,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
}

impl SourceLocation {
    pub fn from_span(
        source_name: impl Into<String>,
        file_path: Option<PathBuf>,
        span: TextSpan,
    ) -> Self {
        Self {
            source_name: source_name.into(),
            file_path,
            start_line: span.line_start,
            start_column: 0,
            end_line: Some(span.line_end),
            end_column: None,
        }
    }

    pub fn source_trace(&self) -> String {
        match self.end_line {
            Some(end) if end != self.start_line => {
                format!("{}:{}-{}", self.source_name, self.start_line, end)
            }
            _ => format!("{}:{}", self.source_name, self.start_line),
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
    pub file_path: Option<PathBuf>,
    pub text: String,
    pub lines: Vec<SourceLine>,
}

impl SourceDocument {
    pub fn new(source_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self::with_path(source_name, None::<PathBuf>, text)
    }

    pub fn with_path(
        source_name: impl Into<String>,
        file_path: Option<impl Into<PathBuf>>,
        text: impl Into<String>,
    ) -> Self {
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
        }
        Self {
            source_name: source_name.into(),
            file_path: file_path.map(Into::into),
            text,
            lines,
        }
    }
}
