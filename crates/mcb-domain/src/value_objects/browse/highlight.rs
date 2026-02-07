use serde::{Deserialize, Serialize};

/// Highlight category for code tokens
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HighlightCategory {
    /// Language keyword (e.g., if, while, return)
    Keyword,
    /// String literal
    String,
    /// Code comment
    Comment,
    /// Function or method name
    Function,
    /// Variable or identifier
    Variable,
    /// Type name or type annotation
    Type,
    /// Numeric literal
    Number,
    /// Operator symbol (e.g., +, -, *)
    Operator,
    /// Punctuation (e.g., brackets, semicolons)
    Punctuation,
    /// Other token type
    Other,
}

/// Represents a span of code with a specific highlight category.
///
/// Defines a range within source code and the category of syntax highlighting
/// to apply to that range.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Starting byte offset of the span in the source code
    pub start: usize,
    /// Ending byte offset of the span in the source code
    pub end: usize,
    /// The highlight category for this span
    pub category: HighlightCategory,
}

/// Complete result of syntax highlighting for source code.
///
/// Contains the original source code, a collection of highlight spans
/// that define syntax highlighting regions, and the programming language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightedCode {
    /// The original source code text
    pub original: String,
    /// Collection of highlight spans defining syntax highlighting regions
    pub spans: Vec<HighlightSpan>,
    /// Programming language of the source code
    pub language: String,
}

impl HighlightedCode {
    /// Creates a new `HighlightedCode` instance.
    ///
    /// # Arguments
    ///
    /// * `original` - The original source code text
    /// * `spans` - Collection of highlight spans defining syntax highlighting regions
    /// * `language` - Programming language of the source code
    ///
    /// # Returns
    ///
    /// A new `HighlightedCode` instance with the provided values.
    pub fn new(original: String, spans: Vec<HighlightSpan>, language: String) -> Self {
        Self {
            original,
            spans,
            language,
        }
    }
}
