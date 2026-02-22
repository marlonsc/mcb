//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
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

use std::collections::{HashMap, HashSet};

use mcb_domain::constants::search::{
    BM25_TOKEN_MIN_LENGTH, HYBRID_SEARCH_BM25_B, HYBRID_SEARCH_BM25_K1,
};
use mcb_domain::entities::CodeChunk;

/// BM25 parameters for tuning the algorithm
#[derive(Debug, Clone)]
pub struct BM25Params {
    /// k1 parameter (term frequency saturation)
    /// Higher values increase the importance of term frequency
    /// Typical range: 1.2 to 2.0
    pub k1: f64,
    /// b parameter (document length normalization)
    /// 0.0 = no normalization, 1.0 = full normalization
    /// Typical value: 0.75
    pub b: f64,
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
    avg_doc_len: f64,
    /// BM25 parameters
    params: BM25Params,
}

impl BM25Scorer {
    /// Create a new BM25 scorer from a collection of documents
    ///
    /// Builds the document frequency index from the provided documents.
    #[must_use]
    pub fn new(documents: &[CodeChunk], params: BM25Params) -> Self {
        let total_docs = documents.len();
        let mut document_freq = HashMap::new();
        let mut total_length = 0.0;

        // Calculate document frequencies and total length
        for doc in documents {
            let tokens = Self::tokenize(&doc.content);
            let doc_length = tokens.len() as f64;
            total_length += doc_length;

            // Count unique terms in this document
            let unique_terms: HashSet<_> = tokens.into_iter().collect();
            for term in unique_terms {
                *document_freq.entry(term).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if total_docs > 0 {
            total_length / total_docs as f64
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
    #[must_use]
    pub fn score(&self, document: &CodeChunk, query: &str) -> f64 {
        let query_terms = Self::tokenize(query);
        self.score_with_tokens(document, &query_terms)
    }

    /// Score a document with pre-tokenized query terms (optimized for batch operations)
    ///
    /// This method avoids re-tokenizing the query for each document, improving performance
    /// when scoring multiple documents against the same query.
    #[must_use]
    pub fn score_with_tokens(&self, document: &CodeChunk, query_terms: &[String]) -> f64 {
        let doc_terms = Self::tokenize(&document.content);
        let doc_length = doc_terms.len() as f64;

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
            let tf = doc_term_freq.get(query_term.as_str()).copied().unwrap_or(0) as f64;
            let df = self.document_freq.get(query_term).copied().unwrap_or(0) as f64;

            if df > 0.0 && tf > 0.0 {
                // IDF calculation using Lucene/Elasticsearch variant that ensures positive IDF
                // This avoids zero/negative IDF when terms appear in half or more documents
                let idf = if self.total_docs > 1 {
                    // Lucene BM25 IDF: ln(1 + (N - n + 0.5) / (n + 0.5))
                    (1.0 + (self.total_docs as f64 - df + 0.5) / (df + 0.5)).ln()
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
    #[must_use]
    pub fn score_batch(&self, documents: &[&CodeChunk], query: &str) -> Vec<f64> {
        let query_terms = Self::tokenize(query);
        documents
            .iter()
            .map(|doc| self.score_with_tokens(doc, &query_terms))
            .collect()
    }

    /// Tokenize text into terms
    ///
    /// Performs lowercase normalization and splits on whitespace, punctuation,
    /// and underscores (for `snake_case` identifiers).
    /// Filters out tokens shorter than `BM25_TOKEN_MIN_LENGTH`.
    #[must_use]
    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() > BM25_TOKEN_MIN_LENGTH)
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    /// Get the total number of indexed documents
    #[must_use]
    pub fn total_docs(&self) -> usize {
        self.total_docs
    }

    /// Get the number of unique terms in the index
    #[must_use]
    pub fn unique_terms(&self) -> usize {
        self.document_freq.len()
    }

    /// Get the average document length
    #[must_use]
    pub fn avg_doc_len(&self) -> f64 {
        self.avg_doc_len
    }

    /// Get the BM25 parameters
    #[must_use]
    pub fn params(&self) -> &BM25Params {
        &self.params
    }
}
