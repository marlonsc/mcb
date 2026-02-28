//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::block_on::validate_block_on_usage;
use super::blocking::validate_blocking_in_async;
use super::mutex::validate_mutex_types;
use super::spawn::validate_spawn_patterns;

crate::create_validator!(
    AsyncPatternValidator,
    "async_patterns",
    "Validates async patterns (blocking calls, mutex types, spawn patterns)",
    AsyncViolation,
    [
        validate_blocking_in_async,
        validate_block_on_usage,
        validate_mutex_types,
        validate_spawn_patterns,
    ]
);
