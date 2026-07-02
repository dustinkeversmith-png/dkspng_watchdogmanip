use crate::navigation::target::NavigationTarget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeNode {
    pub id: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasDefinition {
    pub name: String,
    pub scope_id: String,
    pub target: NavigationTarget,
    pub description: Option<String>,
}

impl AliasDefinition {
    pub fn new(scope_id: impl Into<String>, name: impl Into<String>, target: NavigationTarget) -> Self {
        Self { scope_id: scope_id.into(), name: name.into(), target, description: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub name: String,
    pub scope_id: String,
    pub target: NavigationTarget,
}
