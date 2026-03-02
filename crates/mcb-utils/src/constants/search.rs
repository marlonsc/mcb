//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Search and BM25 algorithmic constants -- Single Source of Truth
//!
//! These are mathematical/algorithmic invariants, NOT operational config.

/// BM25 k1 parameter (term frequency saturation)
pub const HYBRID_SEARCH_BM25_K1: f64 = 1.2;
/// BM25 b parameter (document length normalization)
pub const HYBRID_SEARCH_BM25_B: f64 = 0.75;
/// BM25 token minimum length filter
pub const BM25_TOKEN_MIN_LENGTH: usize = 2;
/// BM25 weight in hybrid search (40% BM25)
pub const HYBRID_SEARCH_BM25_WEIGHT: f64 = 0.4;
/// Semantic weight in hybrid search (60% semantic)
pub const HYBRID_SEARCH_SEMANTIC_WEIGHT: f64 = 0.6;
/// Maximum candidates for hybrid search
pub const HYBRID_SEARCH_MAX_CANDIDATES: usize = 100;

/// Multiplier for hybrid search candidate retrieval (semantic * N)
pub const HYBRID_SEARCH_MULTIPLIER: usize = 3;

/// Over-fetch multiplier for search filtering
pub const SEARCH_OVERFETCH_MULTIPLIER: usize = 2;

// ============================================================================
// RRF (Reciprocal Rank Fusion)
// ============================================================================

/// RRF k parameter for hybrid result fusion
pub const RRF_K: f32 = 60.0;

/// RRF score numerator (1.0 / (rank + k))
pub const RRF_SCORE_NUMERATOR: f32 = 1.0;

/// Maximum possible RRF score multiplier (2 search streams)
pub const RRF_MAX_SCORE_STREAMS: f32 = 2.0;

/// Normalized RRF score ceiling
pub const RRF_NORMALIZED_MAX: f32 = 1.0;
