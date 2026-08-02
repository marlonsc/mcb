//! Infrastructure constants.
//!
//! Files named `constants.rs` are EXEMPT from magic-number checks
//! because they are the correct place to define numeric literals.

/// Maximum concurrent database connections.
pub const MAX_DB_CONNECTIONS: u32 = 100;

/// Default request timeout in milliseconds.
pub const DEFAULT_TIMEOUT_MS: u64 = 30000;

/// Maximum allowed payload size in bytes (10 MB).
pub const MAX_PAYLOAD_SIZE: usize = 10485760;

/// Cache TTL in seconds (24 hours).
pub const CACHE_TTL_SECS: u64 = 86400;

/// Rate limiting: max requests per minute.
pub const RATE_LIMIT_RPM: u32 = 60;
