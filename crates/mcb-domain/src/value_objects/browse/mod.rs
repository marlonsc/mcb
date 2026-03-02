//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#value-objects)
//!
//! Browse value objects for code navigation and file tree representation.

mod collection;
mod file;
mod highlight;
mod node;
/// HTML renderer and highlight category mapping (pure domain utilities).
pub mod renderer;
mod tree;

pub use collection::CollectionInfo;
pub use file::FileInfo;
pub use highlight::{HighlightCategory, HighlightSpan, HighlightedCode};
pub use node::FileNode;
pub use renderer::{HIGHLIGHT_NAMES, HtmlRenderer, map_highlight_to_category};
pub use tree::FileTreeNode;
