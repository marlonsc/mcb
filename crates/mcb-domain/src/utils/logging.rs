//!
//! When to log full error details (off / debug / trace).

/// Level at which full error details are logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorDetailsLevel {
    /// No error detail; only short messages.
    Off,
    /// Detail emitted at debug level.
    #[default]
    Debug,
    /// Detail emitted at trace level.
    Trace,
}

/// Parses config string to `ErrorDetailsLevel`.
#[must_use]
pub fn parse_error_details_level(s: &str) -> ErrorDetailsLevel {
    match s.to_lowercase().as_str() {
        "off" => ErrorDetailsLevel::Off,
        "trace" => ErrorDetailsLevel::Trace,
        _ => ErrorDetailsLevel::Debug,
    }
}
