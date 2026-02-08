//! Hybrid Search Providers
//!
//! This module provides hybrid search functionality that combines BM25 text-based
//! ranking with semantic (vector) search for improved relevance.
//!
//! ## Architecture
//!
//! ```text
//! Query Input
//!     |
//!     v
//! +-------------------+     +-------------------+
//! | BM25 Scorer       |     | Vector Store      |
//! | (lexical match)   |     | (semantic match)  |
//! +-------------------+     +-------------------+
//!           |                       |
//!           v                       v
//!     BM25 Score              Semantic Score
//!           |                       |
//!           +----------+------------+
//!                      |
//!                      v
//!              Score Fusion
//!     hybrid = bm25_weight * bm25 + semantic_weight * semantic
//!                      |
//!                      v
//!              Ranked Results
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use mcb_providers::hybrid_search::HybridSearchEngine;
//! use mcb_domain::ports::providers::HybridSearchProvider;
//!
//! // Create engine with default weights (40% BM25, 60% semantic)
//! let engine = HybridSearchEngine::new();
//!
//! // Index code chunks for BM25 scoring
//! // engine.index_chunks("my-project", &code_chunks).await?;
//!
//! // Get semantic results from vector store
//! // let semantic_results = vector_store.search(...).await?;
//!
//! // Combine with BM25 for hybrid ranking
//! // let results = engine.search("my-project", "auth middleware", semantic_results, 10).await?;
//! ```
//!
//! ## Providers
//!
//! | Provider | Description | Use Case |
//! |----------|-------------|----------|
//! | `HybridSearchEngine` | Full BM25 + semantic hybrid | Production search |
//!
//! ## BM25 Algorithm
//!
//! BM25 (Best Matching 25) scores documents based on term frequency and
//! inverse document frequency:
//!
//! - **High scores**: Documents with query terms that are rare in the corpus
//! - **Low scores**: Documents with common terms or no matching terms
//!
//! Parameters:
//! - `k1`: Term frequency saturation (default: 1.2)
//! - `b`: Document length normalization (default: 0.75)

pub mod bm25;
pub mod engine;

// Re-export main types
pub use bm25::{BM25Params, BM25Scorer};
pub use engine::HybridSearchEngine;
