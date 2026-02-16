//! Constants for the duplication detection module.

// ============================================================================
// Rabin-Karp Rolling Hash Parameters
// ============================================================================

/// Base for the Rabin-Karp rolling hash (small prime).
pub const RABIN_KARP_BASE: u64 = 31;

/// Modulus for the Rabin-Karp rolling hash (large prime for collision resistance).
pub const RABIN_KARP_MODULUS: u64 = 1_000_000_007;

// ============================================================================
// Token Normalization Placeholders
// ============================================================================

/// Placeholder for normalized identifiers in Type-2 (renamed) clone detection.
pub const NORMALIZED_IDENTIFIER: &str = "$ID";

/// Placeholder for normalized literals in Type-2 (renamed) clone detection.
pub const NORMALIZED_LITERAL: &str = "$LIT";

// ============================================================================
// Character Classification Sets
// ============================================================================

/// Characters classified as operators in token classification.
pub const OPERATOR_CHARS: &str = "+-*%=<>!&|^~";

/// Characters classified as punctuation in token classification.
pub const PUNCTUATION_CHARS: &str = "(){}[];:,.?";

// ============================================================================
// Default Duplication Thresholds
// ============================================================================

/// Default minimum lines for a clone to be reported.
pub const DEFAULT_MIN_LINES: usize = 6;

/// Default minimum tokens for a clone to be reported.
pub const DEFAULT_MIN_TOKENS: usize = 50;

/// Default similarity threshold for duplicate detection.
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.80;

/// Default maximum gap size for gapped (Type-3) clones.
pub const DEFAULT_MAX_GAP_SIZE: usize = 5;

/// Strict mode: minimum lines.
pub const STRICT_MIN_LINES: usize = 4;

/// Strict mode: minimum tokens.
pub const STRICT_MIN_TOKENS: usize = 30;

/// Strict mode: similarity threshold.
pub const STRICT_SIMILARITY_THRESHOLD: f64 = 0.90;

/// Lenient mode: minimum lines.
pub const LENIENT_MIN_LINES: usize = 10;

/// Lenient mode: minimum tokens.
pub const LENIENT_MIN_TOKENS: usize = 100;

/// Lenient mode: similarity threshold.
pub const LENIENT_SIMILARITY_THRESHOLD: f64 = 0.70;

// ============================================================================
// Default Languages & Exclusion Patterns
// ============================================================================

/// Languages analyzed by default.
pub const DEFAULT_LANGUAGES: &[&str] = &["rust", "python", "javascript", "typescript"];

/// Glob patterns excluded from duplication analysis by default.
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**",
    "**/vendor/**",
];
