//! Logging bridge: infrastructure-side logging implementations.
//!
//! Provides `tracing_log_fn` that wraps the `tracing` crate to implement the
//! domain-level `LogFn` signature. Registered at startup via
//! `mcb_domain::infra::logging::set_log_fn`.

use mcb_domain::ports::LogLevel;

/// A log function that forwards domain log events to `tracing`.
///
/// Register at startup:
/// ```ignore
/// mcb_domain::infra::logging::set_log_fn(mcb_infrastructure::logging::tracing_log_fn);
/// ```
pub fn tracing_log_fn(
    level: LogLevel,
    context: &str,
    message: &str,
    detail: Option<&dyn std::fmt::Display>,
) {
    macro_rules! emit {
        ($lvl:expr) => {
            if let Some(d) = detail {
                tracing::event!($lvl, context = %context, detail = %d, "{}", message);
            } else {
                tracing::event!($lvl, context = %context, "{}", message);
            }
        };
    }

    match level {
        LogLevel::Error => emit!(tracing::Level::ERROR),
        LogLevel::Warn => emit!(tracing::Level::WARN),
        LogLevel::Info => emit!(tracing::Level::INFO),
        LogLevel::Debug => emit!(tracing::Level::DEBUG),
        LogLevel::Trace => emit!(tracing::Level::TRACE),
    }
}
