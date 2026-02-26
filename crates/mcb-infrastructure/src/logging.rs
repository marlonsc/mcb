//! Logging adapter: forwards domain `OperationLogger` and `set_log_fn` callback to tracing.
//!
//! Single entry point for all MCB logging via mcb-domain port; Loco/tracing remain the
//! implementation detail. Register at startup with `mcb_domain::infra::logging::set_log_fn(tracing_log_fn)`.

use mcb_domain::ports::{LogLevel, OperationLogger};

/// Log function suitable for `mcb_domain::infra::logging::set_log_fn`.
/// Forwards level, context, message and optional detail to `tracing::event!`.
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

/// Adapter that implements domain `OperationLogger` by forwarding to tracing.
/// Can be registered in DI when an `Arc<dyn OperationLogger>` is required.
#[derive(Debug)]
pub struct TracingOperationLogger;

impl TracingOperationLogger {
    /// Creates a new tracing-backed operation logger.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TracingOperationLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationLogger for TracingOperationLogger {
    fn log(
        &self,
        level: LogLevel,
        context: &str,
        message: &str,
        detail: Option<&dyn std::fmt::Display>,
    ) {
        tracing_log_fn(level, context, message, detail);
    }
}
