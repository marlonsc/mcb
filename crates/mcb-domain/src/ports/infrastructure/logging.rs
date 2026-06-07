//! Logging ports.

/// Log level for the unified `log` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Critical error that requires immediate attention.
    Error,
    /// Potentially problematic situation that doesn't prevent operation.
    Warn,
    /// Significant informational event for monitoring purposes.
    Info,
    /// Detailed information useful for debugging.
    Debug,
    /// Extremely fine-grained information for tracing execution flow.
    Trace,
}

/// Operation logger interface: one method for all levels, optional detail.
pub trait OperationLogger: Send + Sync {
    /// Logs a message with a specific level, context, and optional detail.
    ///
    /// # Parameters
    /// - `level`: The severity level of the log message.
    /// - `context`: The name of the module or component generating the log.
    /// - `message`: The primary log message text.
    /// - `detail`: An optional displayable value providing extra context (e.g., a struct).
    fn log(
        &self,
        level: LogLevel,
        context: &str,
        message: &str,
        detail: Option<&dyn std::fmt::Display>,
    );
}
