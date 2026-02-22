//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! VCS impact analysis constants.

/// Weight applied to the file-count term in impact score calculation.
pub const IMPACT_FILE_COUNT_WEIGHT: f64 = 10.0;

/// Weight applied to the change-count term in impact score calculation.
pub const IMPACT_CHANGE_COUNT_WEIGHT: f64 = 5.0;

/// Maximum impact score (clamped upper bound).
pub const MAX_IMPACT_SCORE: f64 = 100.0;
