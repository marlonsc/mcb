//! Centralized public API surface for mcb-providers.
//! Re-exports only; module declarations stay in lib.rs.

pub use crate::database::migration;
pub use crate::hybrid_search::HybridSearchEngine;

pub use mcb_domain::error::{Error, Result};
pub use mcb_domain::ports::CryptoProvider;
pub use mcb_domain::ports::{
    CacheProvider, ComplexityAnalyzer, DeadCodeDetector, EmbeddingProvider, HybridSearchProvider,
    LanguageChunkingProvider, TdgScorer, VcsProvider, VectorStoreProvider,
};
