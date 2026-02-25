//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Code Duplication Detection (Phase 5)
//!
//! Provides clone detection using a hybrid approach:
//! 1. **Token fingerprinting** (Rabin-Karp) for fast initial detection
//! 2. **AST similarity** (Tree-sitter) for accurate verification
//!
//! ## Clone Types
//!
//! | Type | Rule ID | Description |
//! | ------ | --------- | ------------- |
//! | Type 1 | DUP001 | Exact clones (100% identical) |
//! | Type 2 | DUP002 | Renamed clones (identifiers changed) |
//! | Type 3 | DUP003 | Gapped clones (small modifications) |
//! | Type 4 | DUP004 | Semantic clones (functionally similar) |
//!
//! ## Usage
//!
//! ```text
//! let analyzer = DuplicationAnalyzer::new();
//! let violations = analyzer.analyze_files(&[path1, path2])?;
//!
//! for violation in violations {
//!     println!("{}: {} ({}% similar)",
//!         violation.id(),
//!         violation.message(),
//!         (violation.similarity * 100.0) as u32
//!     );
//! }
//! ```

pub mod analyzer;
pub mod detector;
pub mod fingerprint;
pub mod thresholds;
pub mod violation;

pub use self::analyzer::{DuplicationAnalyzer, DuplicationStats};
pub use self::detector::{CloneCandidate, CloneDetector, tokenize_source};
pub use self::fingerprint::{
    Fingerprint, FingerprintLocation, FingerprintMatch, Token, TokenFingerprinter,
};
pub use self::thresholds::{DuplicationThresholds, DuplicationType};
pub use self::violation::DuplicationViolation;
