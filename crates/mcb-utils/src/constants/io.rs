//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! I/O and buffer size constants.

/// Buffer size for file reading operations (8 KiB).
pub const FILE_READ_BUFFER_SIZE: usize = 8192;

/// Number of characters shown when masking IDs for logging.
pub const MASKED_ID_PREFIX_LENGTH: usize = 8;
