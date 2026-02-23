//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Time validation and boundary constants.

/// Minimum valid Unix timestamp (2020-01-01 00:00:00 UTC).
pub const TIMESTAMP_MIN_BOUNDARY: i64 = 1_577_836_800;

/// Maximum valid Unix timestamp (2100-01-01 00:00:00 UTC).
pub const TIMESTAMP_MAX_BOUNDARY: i64 = 4_102_444_800;
