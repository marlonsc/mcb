//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#value-objects)
//!
//! Browse value objects for code navigation and file tree representation.

mod collection;
mod file;
mod highlight;
mod node;
mod tree;

pub use collection::CollectionInfo;
pub use file::FileInfo;
pub use highlight::{
    HIGHLIGHT_NAMES, HighlightCategory, HighlightSpan, HighlightedCode, map_highlight_to_category,
};
pub use node::FileNode;
pub use tree::FileTreeNode;
