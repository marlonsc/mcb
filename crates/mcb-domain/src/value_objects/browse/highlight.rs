use serde::{Deserialize, Serialize};

/// Highlight category for code tokens
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HighlightCategory {
    Keyword,
    String,
    Comment,
    Function,
    Variable,
    Type,
    Number,
    Operator,
    Punctuation,
    Other,
}

/// Highlighted code span
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub category: HighlightCategory,
}

/// Highlighted code result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightedCode {
    pub original: String,
    pub spans: Vec<HighlightSpan>,
    pub language: String,
}

impl HighlightedCode {
    pub fn new(original: String, spans: Vec<HighlightSpan>, language: String) -> Self {
        Self {
            original,
            spans,
            language,
        }
    }
}
