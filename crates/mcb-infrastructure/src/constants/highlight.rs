//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Syntax highlighting constants

/// Tree-sitter highlight capture names (order must match `HighlightConfiguration`)
pub const HIGHLIGHT_NAMES: [&str; 13] = [
    "keyword",
    "function",
    "string",
    "comment",
    "type",
    "variable",
    "constant",
    "operator",
    "attribute",
    "number",
    "punctuation",
    "property",
    "tag",
];
