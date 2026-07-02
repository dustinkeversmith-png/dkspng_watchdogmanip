pub mod error;
pub mod index;
pub mod model;
pub mod parser;
pub mod fs_indexer;

pub use self::error::{ContextError, Result};
pub use self::index::{ContextIndex, ContextTreeNode};
pub use self::model::*;
pub use self::parser::{build_index_from_document, ParseConfig};
pub use self::fs_indexer::*;
