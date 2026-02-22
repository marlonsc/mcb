//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
/// Constant value for `DAEMON_CHECK_INTERVAL_SECS`.
pub const DAEMON_CHECK_INTERVAL_SECS: u64 = 30;
/// Constant value for `DAEMON_RESTART_DELAY_SECS`.
pub const DAEMON_RESTART_DELAY_SECS: u64 = 5;
/// Constant value for `DAEMON_MAX_RESTART_ATTEMPTS`.
pub const DAEMON_MAX_RESTART_ATTEMPTS: u32 = 3;
/// Constant value for `GRACEFUL_SHUTDOWN_TIMEOUT_SECS`.
pub const GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 30;
/// Constant value for `FORCE_SHUTDOWN_TIMEOUT_SECS`.
pub const FORCE_SHUTDOWN_TIMEOUT_SECS: u64 = 10;
/// Constant value for `SIGNAL_POLL_INTERVAL_MS`.
pub const SIGNAL_POLL_INTERVAL_MS: u64 = 100;
