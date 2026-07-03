use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseHierarchyNode {
    pub command_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub hierarchy_path: Vec<String>,
}
