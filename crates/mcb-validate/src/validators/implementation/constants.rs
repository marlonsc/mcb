//! Constants for implementation quality validators.

/// Hardcoded return pattern IDs and descriptions: (`pattern_id`, description).
pub const HARDCODED_RETURN_PATTERNS: &[(&str, &str)] = &[
    ("IMPL001.return_true", "true"),
    ("IMPL001.return_false", "false"),
    ("IMPL001.return_zero", "0"),
    ("IMPL001.return_one", "1"),
    ("IMPL001.return_empty_string", "empty string"),
    ("IMPL001.return_hardcoded_string", "hardcoded string"),
];

/// File names to skip in hardcoded return detection.
pub const STUB_SKIP_FILE_KEYWORDS: &[&str] = &["null", "fake", "constants.rs"];
