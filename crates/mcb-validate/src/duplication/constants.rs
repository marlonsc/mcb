//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Re-exports duplication constants from the central constants module.
//!
//! All duplication thresholds, keywords, and patterns live in
//! [`crate::constants::duplication`].

pub use crate::constants::duplication::{
    DEFAULT_EXCLUDE_PATTERNS,
    DEFAULT_LANGUAGES,
    DEFAULT_MAX_GAP_SIZE,
    DEFAULT_MIN_LINES,
    DEFAULT_MIN_TOKENS,
    DEFAULT_SIMILARITY_THRESHOLD,
    DUPLICATION_KEYWORDS,
    LENIENT_MIN_LINES,
    LENIENT_MIN_TOKENS,
    LENIENT_SIMILARITY_THRESHOLD,
    NORMALIZED_IDENTIFIER,
    NORMALIZED_LITERAL,
    OPERATOR_CHARS,
    PUNCTUATION_CHARS,
    RABIN_KARP_BASE,
    RABIN_KARP_MODULUS,
    STRICT_MIN_LINES,
    STRICT_MIN_TOKENS,
    STRICT_SIMILARITY_THRESHOLD,
};
