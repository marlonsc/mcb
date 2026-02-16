//! SOLID validator constants.
//!
//! Thresholds and limits for Single Responsibility, Open-Closed, and
//! related SOLID principle checks.

/// Max unrelated structs in a single file before SRP warning.
pub const MAX_UNRELATED_STRUCTS_PER_FILE: usize = 5;

/// Min string-based match arms before OCP dispatch warning.
pub const MIN_STRING_MATCH_ARMS_FOR_DISPATCH: usize = 3;

/// Min names needed for relationship check.
pub const MIN_NAMES_FOR_RELATION_CHECK: usize = 2;

/// Min shared prefix/suffix length for relationship detection.
pub const MIN_AFFIX_LENGTH: usize = 3;

/// Max shared prefix/suffix length for relationship detection.
pub const MAX_AFFIX_LENGTH: usize = 10;

/// Min word length for semantic comparison in CamelCase splitting.
pub const MIN_WORD_LENGTH_FOR_COMPARISON: usize = 4;
