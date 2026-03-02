//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Event bus and messaging constants.

/// Default capacity for event bus channels.
pub const EVENT_BUS_DEFAULT_CAPACITY: usize = 1024;
/// Constant value for `EVENT_BUS_BUFFER_SIZE`.
pub const EVENT_BUS_BUFFER_SIZE: usize = 1000;
/// Constant value for `EVENT_BUS_CONNECTION_TIMEOUT_MS`.
pub const EVENT_BUS_CONNECTION_TIMEOUT_MS: u64 = 5000;
/// Constant value for `EVENT_BUS_MAX_RECONNECT_ATTEMPTS`.
pub const EVENT_BUS_MAX_RECONNECT_ATTEMPTS: u32 = 5;
