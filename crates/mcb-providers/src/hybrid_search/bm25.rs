//! BM25 text ranking algorithm implementation
//!
//! BM25 (Best Matching 25) is a ranking function used for information retrieval.
//! It ranks documents based on the query terms appearing in each document.
//!
//! # Algorithm
//!
//! BM25 score = sum(IDF(qi) * (f(qi,D) * (k1+1)) / (f(qi,D) + k1 * (1 - b + b * |D|/avgdl)))
//!
//! Where:
//! - qi = query term
//! - f(qi,D) = frequency of qi in document D
//! - |D| = document length
//! - avgdl = average document length
//! - k1, b = tuning parameters

use mcb_domain::entities::CodeChunk;
use std::collections::{HashMap, HashSet};

use crate::constants::{BM25_TOKEN_MIN_LENGTH, HYBRID_SEARCH_BM25_B, HYBRID_SEARCH_BM25_K1};

/// BM25 parameters for tuning the algorithm
#[derive(Debug, Clone)]
pub struct BM25Params {
    /// k1 parameter (term frequency saturation)
    /// Higher values increase the importance of term frequency
    /// Typical range: 1.2 to 2.0
    pub k1: f32,
    /// b parameter (document length normalization)
    /// 0.0 = no normalization, 1.0 = full normalization
    /// Typical value: 0.75
    pub b: f32,
}

impl Default for BM25Params {
    fn default() -> Self {
        Self {
            k1: HYBRID_SEARCH_BM25_K1,
            b: HYBRID_SEARCH_BM25_B,
        }
    }
}

/// BM25 scorer for text-based ranking
///
/// This scorer maintains document frequency statistics and provides
/// efficient scoring of documents against queries.
#[derive(Debug)]
pub struct BM25Scorer {
    /// Document frequencies for each term (how many documents contain each term)
    document_freq: HashMap<String, usize>,
    /// Total number of documents
    total_docs: usize,
    /// Average document length (in tokens)
    avg_doc_len: f32,
    /// BM25 parameters
    params: BM25Params,
}

impl BM25Scorer {
    /// Create a new BM25 scorer from a collection of documents
    ///
    /// Builds the document frequency index from the provided documents.
    pub fn new(documents: &[CodeChunk], params: BM25Params) -> Self {
        let total_docs = documents.len();
        let mut document_freq = HashMap::new();
        let mut total_length = 0.0;

        // Calculate document frequencies and total length
        for doc in documents {
            let tokens = Self::tokenize(&doc.content);
            let doc_length = tokens.len() as f32;
            total_length += doc_length;

            // Count unique terms in this document
            let unique_terms: HashSet<_> = tokens.into_iter().collect();
            for term in unique_terms {
                *document_freq.entry(term).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if total_docs > 0 {
            total_length / total_docs as f32
        } else {
            0.0
        };

        Self {
            document_freq,
            total_docs,
            avg_doc_len,
            params,
        }
    }

    /// Score a document against a query using BM25
    pub fn score(&self, document: &CodeChunk, query: &str) -> f32 {
        let query_terms = Self::tokenize(query);
        self.score_with_tokens(document, &query_terms)
    }

    /// Score a document with pre-tokenized query terms (optimized for batch operations)
    ///
    /// This method avoids re-tokenizing the query for each document, improving performance
    /// when scoring multiple documents against the same query.
    pub fn score_with_tokens(&self, document: &CodeChunk, query_terms: &[String]) -> f32 {
        let doc_terms = Self::tokenize(&document.content);
        let doc_length = doc_terms.len() as f32;

        // Early return for empty documents
        if doc_length == 0.0 || self.avg_doc_len == 0.0 {
            return 0.0;
        }

        let mut score = 0.0;
        let mut doc_term_freq: HashMap<&str, usize> = HashMap::new();

        // Count term frequencies in document
        for term in &doc_terms {
            *doc_term_freq.entry(term.as_str()).or_insert(0) += 1;
        }

        // Calculate BM25 score for each query term
        for query_term in query_terms {
            let tf = doc_term_freq.get(query_term.as_str()).copied().unwrap_or(0) as f32;
            let df = self.document_freq.get(query_term).copied().unwrap_or(0) as f32;

            if df > 0.0 && tf > 0.0 {
                // IDF calculation
                let idf = if self.total_docs > 1 {
                    // Standard BM25 IDF for multiple documents
                    ((self.total_docs as f32 - df + 0.5) / (df + 0.5)).ln()
                } else {
                    // Simplified IDF for single document (always positive)
                    1.0
                };

                // Term frequency normalization
                let tf_normalized = (tf * (self.params.k1 + 1.0))
                    / (tf
                        + self.params.k1
                            * (1.0 - self.params.b
                                + self.params.b * doc_length / self.avg_doc_len));

                score += idf * tf_normalized;
            }
        }

        score
    }

    /// Score multiple documents with a single tokenization pass (batch optimization)
    ///
    /// This is more efficient than calling `score()` for each document because
    /// the query is tokenized only once.
    pub fn score_batch(&self, documents: &[&CodeChunk], query: &str) -> Vec<f32> {
        let query_terms = Self::tokenize(query);
        documents
            .iter()
            .map(|doc| self.score_with_tokens(doc, &query_terms))
            .collect()
    }

    /// Tokenize text into terms
    ///
    /// Performs lowercase normalization and splits on whitespace and punctuation.
    /// Filters out tokens shorter than `BM25_TOKEN_MIN_LENGTH`.
    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && s.len() > BM25_TOKEN_MIN_LENGTH)
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the total number of indexed documents
    pub fn total_docs(&self) -> usize {
        self.total_docs
    }

    /// Get the number of unique terms in the index
    pub fn unique_terms(&self) -> usize {
        self.document_freq.len()
    }

    /// Get the average document length
    pub fn avg_doc_len(&self) -> f32 {
        self.avg_doc_len
    }

    /// Get the BM25 parameters
    pub fn params(&self) -> &BM25Params {
        &self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcb_domain::value_objects::types::Language;

    fn create_test_chunk(content: &str, file_path: &str, start_line: usize) -> CodeChunk {
        CodeChunk {
            id: format!("{}:{}", file_path, start_line),
            content: content.to_string(),
            file_path: file_path.to_string(),
            start_line,
            end_line: start_line + content.lines().count(),
            language: Language::Rust,
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_tokenize() {
        let tokens = BM25Scorer::tokenize("fn hello_world() { println!(\"Hello, World!\"); }");
        assert!(tokens.contains(&"hello_world".to_string()));
        assert!(tokens.contains(&"println".to_string()));
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        // Short tokens should be filtered
        assert!(!tokens.contains(&"fn".to_string())); // len = 2, filtered by BM25_TOKEN_MIN_LENGTH
    }

    #[test]
    fn test_bm25_scorer_creation() {
        let chunks = vec![
            create_test_chunk("fn authenticate_user() {}", "auth.rs", 1),
            create_test_chunk("fn validate_password() {}", "auth.rs", 10),
            create_test_chunk("fn hash_password() {}", "crypto.rs", 1),
        ];

        let scorer = BM25Scorer::new(&chunks, BM25Params::default());

        assert_eq!(scorer.total_docs(), 3);
        assert!(scorer.unique_terms() > 0);
        assert!(scorer.avg_doc_len() > 0.0);
    }

    #[test]
    fn test_bm25_scoring() {
        let chunks = vec![
            create_test_chunk("fn authenticate_user(username: String) { validate(username) }", "auth.rs", 1),
            create_test_chunk("fn validate_password(password: String) { hash(password) }", "auth.rs", 10),
            create_test_chunk("fn process_data(data: Vec<u8>) { compress(data) }", "data.rs", 1),
        ];

        let scorer = BM25Scorer::new(&chunks, BM25Params::default());

        // Query about authentication should score auth.rs higher
        let score_auth = scorer.score(&chunks[0], "authenticate user validation");
        let score_password = scorer.score(&chunks[1], "authenticate user validation");
        let score_data = scorer.score(&chunks[2], "authenticate user validation");

        // Auth chunk should score highest for auth-related query
        assert!(score_auth > score_data, "Auth chunk should score higher than data chunk");
    }

    #[test]
    fn test_bm25_batch_scoring() {
        let chunks = vec![
            create_test_chunk("fn search_code() {}", "search.rs", 1),
            create_test_chunk("fn index_code() {}", "index.rs", 1),
        ];

        let scorer = BM25Scorer::new(&chunks, BM25Params::default());
        let chunk_refs: Vec<&CodeChunk> = chunks.iter().collect();

        let scores = scorer.score_batch(&chunk_refs, "search code");

        assert_eq!(scores.len(), 2);
        // First chunk should score higher for "search code" query
        assert!(scores[0] > scores[1]);
    }
}
