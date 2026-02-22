//! Operation logging port: single entry point (level + context + message + optional detail).

/// Log level for the unified `log` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Error level
    Error,
    /// Warning level
    Warn,
    /// Info level
    Info,
    /// Debug level
    Debug,
    /// Trace level
    Trace,
}

/// Operation logger interface: one method for all levels, optional detail.
pub trait OperationLogger: Send + Sync {
    /// Logs at the given level. Message always; detail only when config permits.
    fn log(
        &self,
        level: LogLevel,
        context: &str,
        message: &str,
        detail: Option<&dyn std::fmt::Display>,
    );
}
