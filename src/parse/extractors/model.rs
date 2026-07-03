use crate::parse::boundary::model::BoundaryCandidate;
use crate::parse::seeds::model::DetectedSeed;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub source_name: String,
    pub seed: DetectedSeed,
    pub boundary: BoundaryCandidate,
    pub body: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedCommandParts {
    pub title: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub members: BTreeMap<String, Value>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
}

pub trait CommandExtractor {
    fn name(&self) -> &'static str;
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts;
}
