/// Maximum number of file paths to return when building collection tree
pub const LIST_FILE_PATHS_LIMIT: usize = 10_000;

/// Valid configuration sections for admin config updates
pub const VALID_SECTIONS: &[&str] = &[
    "server",
    "logging",
    "cache",
    "metrics",
    "limits",
    "resilience",
];
