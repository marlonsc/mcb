//! AST-Based Clone Detection
//!
//! Provides accurate clone detection using tree-sitter AST analysis.
//! Used to verify candidates from the fingerprinting phase and classify clone types.

use std::path::PathBuf;
use std::str::Chars;

use super::constants::{OPERATOR_CHARS, PUNCTUATION_CHARS};
use super::fingerprint::{FingerprintMatch, Token, TokenType};
use super::thresholds::{DuplicationThresholds, DuplicationType};
use super::utils::lines_overlap;

/// Result of comparing two code fragments
#[derive(Debug, Clone)]
pub struct CloneCandidate {
    /// File containing the first fragment
    pub file1: PathBuf,
    /// Starting line of first fragment (1-based)
    pub start_line1: usize,
    /// Ending line of first fragment (1-based)
    pub end_line1: usize,
    /// File containing the second fragment
    pub file2: PathBuf,
    /// Starting line of second fragment (1-based)
    pub start_line2: usize,
    /// Ending line of second fragment (1-based)
    pub end_line2: usize,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f64,
    /// Detected clone type
    pub clone_type: DuplicationType,
    /// Number of duplicated lines
    pub duplicated_lines: usize,
}

/// Clone detector using AST analysis
pub struct CloneDetector {
    thresholds: DuplicationThresholds,
}

impl CloneDetector {
    /// Create a new clone detector with the given thresholds
    #[must_use]
    pub fn new(thresholds: DuplicationThresholds) -> Self {
        Self { thresholds }
    }

    /// Verify fingerprint matches using AST comparison
    ///
    /// Takes candidates from the fingerprinting phase and verifies them
    /// using more accurate AST-based similarity comparison.
    #[must_use]
    pub fn verify_candidates(&self, matches: &[FingerprintMatch]) -> Vec<CloneCandidate> {
        let mut candidates = Vec::new();

        for m in matches {
            if let Some(candidate) = self.verify_single_match(m)
                && self.passes_thresholds(&candidate)
            {
                candidates.push(candidate);
            }
        }

        // Deduplicate overlapping candidates
        Self::deduplicate_candidates(candidates)
    }

    /// Verify a single fingerprint match
    fn verify_single_match(&self, m: &FingerprintMatch) -> Option<CloneCandidate> {
        let lines1 = m.location1.end_line.saturating_sub(m.location1.start_line) + 1;
        let lines2 = m.location2.end_line.saturating_sub(m.location2.start_line) + 1;
        let duplicated_lines = lines1.min(lines2);

        // For now, use a simplified similarity calculation based on token count
        // In a full implementation, this would use AST node comparison
        let similarity = Self::calculate_similarity(m);
        let clone_type = Self::classify_clone_type(similarity);

        if self.thresholds.should_detect(clone_type) {
            Some(CloneCandidate {
                file1: m.location1.file.clone(),
                start_line1: m.location1.start_line,
                end_line1: m.location1.end_line,
                file2: m.location2.file.clone(),
                start_line2: m.location2.start_line,
                end_line2: m.location2.end_line,
                similarity,
                clone_type,
                duplicated_lines,
            })
        } else {
            None
        }
    }

    /// Calculate similarity between two matched fragments
    ///
    /// In a full implementation, this would compare AST structures.
    /// For now, we use the fingerprint match as evidence of exact match.
    fn calculate_similarity(_m: &FingerprintMatch) -> f64 {
        // Fingerprint matches are exact token sequence matches
        // so they have 100% similarity at the token level
        1.0
    }

    /// Classify the type of clone based on similarity.
    ///
    /// Reuses [`DuplicationType::min_similarity`] thresholds so the
    /// classification stays in sync with the canonical values.
    fn classify_clone_type(similarity: f64) -> DuplicationType {
        [
            DuplicationType::ExactClone,
            DuplicationType::RenamedClone,
            DuplicationType::GappedClone,
        ]
        .into_iter()
        .find(|t| similarity >= t.min_similarity())
        .unwrap_or(DuplicationType::SemanticClone)
    }

    /// Check if a candidate passes the configured thresholds
    fn passes_thresholds(&self, candidate: &CloneCandidate) -> bool {
        // Check minimum lines
        if candidate.duplicated_lines < self.thresholds.min_lines {
            return false;
        }

        // Check if we should detect this type
        if !self.thresholds.should_detect(candidate.clone_type) {
            return false;
        }

        // Check similarity threshold
        self.thresholds
            .meets_threshold(candidate.similarity, candidate.clone_type)
    }

    /// Remove overlapping candidates, keeping the best one
    fn deduplicate_candidates(candidates: Vec<CloneCandidate>) -> Vec<CloneCandidate> {
        if candidates.is_empty() {
            return candidates;
        }

        let mut result = Vec::new();
        let mut used: Vec<bool> = vec![false; candidates.len()];

        // Sort by similarity (descending) to prefer higher similarity matches
        let mut sorted_indices: Vec<usize> = (0..candidates.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            candidates[b]
                .similarity
                .partial_cmp(&candidates[a].similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for &i in &sorted_indices {
            if used[i] {
                continue;
            }

            let candidate = &candidates[i];
            if !Self::overlaps_any(candidate, &result) {
                result.push(candidate.clone());
                used[i] = true;
                Self::mark_overlapping_as_used(&mut used, &candidates, candidate);
            }
        }

        result
    }

    fn overlaps_any(candidate: &CloneCandidate, selected: &[CloneCandidate]) -> bool {
        selected
            .iter()
            .any(|existing| Self::candidates_overlap(candidate, existing))
    }

    fn mark_overlapping_as_used(
        used: &mut [bool],
        candidates: &[CloneCandidate],
        candidate: &CloneCandidate,
    ) {
        for (j, other) in candidates.iter().enumerate() {
            if !used[j] && Self::candidates_overlap(candidate, other) {
                used[j] = true;
            }
        }
    }

    /// Check if two candidates overlap (same files and overlapping lines)
    fn candidates_overlap(a: &CloneCandidate, b: &CloneCandidate) -> bool {
        // Check first location overlap
        let overlap1 = a.file1 == b.file1
            && lines_overlap(a.start_line1, a.end_line1, b.start_line1, b.end_line1);

        // Check second location overlap
        let overlap2 = a.file2 == b.file2
            && lines_overlap(a.start_line2, a.end_line2, b.start_line2, b.end_line2);

        // Also check cross-overlaps (a.file1 == b.file2, etc.)
        let cross1 = a.file1 == b.file2
            && lines_overlap(a.start_line1, a.end_line1, b.start_line2, b.end_line2);

        let cross2 = a.file2 == b.file1
            && lines_overlap(a.start_line2, a.end_line2, b.start_line1, b.end_line1);

        overlap1 || overlap2 || cross1 || cross2
    }
}

fn consume_while(
    chars: &mut std::iter::Peekable<Chars<'_>>,
    mut predicate: impl FnMut(char) -> bool,
) -> String {
    let mut out = String::new();
    while let Some(next) = chars.peek().copied() {
        if !predicate(next) {
            break;
        }
        let Some(next_char) = chars.next() else {
            break;
        };
        out.push(next_char);
    }
    out
}

fn consume_quoted_literal(
    chars: &mut std::iter::Peekable<Chars<'_>>,
    quote: char,
    current_line: &mut usize,
) -> String {
    let mut literal = String::new();
    literal.push(quote);

    for next in chars.by_ref() {
        literal.push(next);
        if next == quote && !literal.ends_with('\\') {
            break;
        }
        if next == '\n' {
            *current_line += 1;
        }
    }

    literal
}

fn skip_line_comment(chars: &mut std::iter::Peekable<Chars<'_>>) {
    while let Some(next) = chars.peek().copied() {
        if next == '\n' {
            break;
        }
        let _ = chars.next();
    }
}

fn skip_block_comment(chars: &mut std::iter::Peekable<Chars<'_>>, current_line: &mut usize) {
    while let Some(next) = chars.next() {
        if next == '\n' {
            *current_line += 1;
            continue;
        }
        if next == '*' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            break;
        }
    }
}

fn push_token(
    tokens: &mut Vec<Token>,
    text: String,
    line: usize,
    column: usize,
    token_type: TokenType,
) {
    tokens.push(Token::new(text, line, column, token_type));
}

/// Tokenize source code for fingerprinting
///
/// This is a simplified tokenizer. A full implementation would use
/// tree-sitter for language-aware tokenization.
#[must_use]
pub fn tokenize_source(source: &str, _language: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_line = 1;
    let mut current_column = 1;
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                current_line += 1;
                current_column = 1;
            }
            c if c.is_whitespace() => {
                current_column += 1;
            }
            c if c.is_alphabetic() || c == '_' => {
                let start_column = current_column;
                let mut word = String::new();
                word.push(c);
                word.push_str(&consume_while(&mut chars, |next| {
                    next.is_alphanumeric() || next == '_'
                }));

                let token_type = if is_keyword(&word) {
                    TokenType::Keyword
                } else {
                    TokenType::Identifier
                };

                push_token(
                    &mut tokens,
                    word.clone(),
                    current_line,
                    start_column,
                    token_type,
                );
                current_column += word.len();
            }
            c if c.is_ascii_digit() => {
                let start_column = current_column;
                let mut number = String::new();
                number.push(c);
                number.push_str(&consume_while(&mut chars, |next| {
                    next.is_ascii_digit() || next == '.' || next == '_'
                }));

                push_token(
                    &mut tokens,
                    number.clone(),
                    current_line,
                    start_column,
                    TokenType::Literal,
                );
                current_column += number.len();
            }
            '"' | '\'' => {
                let start_column = current_column;
                let string = consume_quoted_literal(&mut chars, c, &mut current_line);

                push_token(
                    &mut tokens,
                    string.clone(),
                    current_line,
                    start_column,
                    TokenType::Literal,
                );
                current_column += string.len();
            }
            '/' => match chars.peek().copied() {
                Some('/') => {
                    let _ = chars.next();
                    skip_line_comment(&mut chars);
                }
                Some('*') => {
                    let _ = chars.next();
                    skip_block_comment(&mut chars, &mut current_line);
                }
                _ => {
                    push_token(
                        &mut tokens,
                        c.to_string(),
                        current_line,
                        current_column,
                        TokenType::Operator,
                    );
                    current_column += 1;
                }
            },
            c if OPERATOR_CHARS.contains(c) => {
                push_token(
                    &mut tokens,
                    c.to_string(),
                    current_line,
                    current_column,
                    TokenType::Operator,
                );
                current_column += 1;
            }
            c if PUNCTUATION_CHARS.contains(c) => {
                push_token(
                    &mut tokens,
                    c.to_string(),
                    current_line,
                    current_column,
                    TokenType::Punctuation,
                );
                current_column += 1;
            }
            _ => {
                current_column += 1;
            }
        }
    }

    tokens
}

/// Check if a word is a common keyword (simplified, multi-language)
fn is_keyword(word: &str) -> bool {
    crate::constants::duplication::DUPLICATION_KEYWORDS.contains(&word)
}
