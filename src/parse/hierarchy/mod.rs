pub mod detector;
pub mod model;
pub mod resolver;
pub mod strategies;

pub use detector::HierarchyDetectorRegistry;
pub use model::HierarchyDetector;
pub use model::*;
pub use resolver::{resolve_hierarchy, resolve_hierarchy_with_detectors};
pub use strategies::{
    bullet_lists::BulletListHierarchyDetector, headings::MarkdownHeadingHierarchyDetector,
    indentation::IndentationHierarchyDetector, numbered_lists::NumberedListHierarchyDetector,
};
