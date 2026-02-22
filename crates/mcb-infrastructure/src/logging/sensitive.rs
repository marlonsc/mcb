//! Shim that delegates to the global `OperationLogger` (set by catalog).
//! Also provides the domain log-facade shim so `mcb_domain::error!` etc. route here.

use std::sync::Arc;
use std::sync::OnceLock;

use mcb_domain::ports::{LogLevel, OperationLogger};

static GLOBAL_LOGGER: OnceLock<Arc<dyn OperationLogger>> = OnceLock::new();

/// Sets the global operation logger. Called from catalog after building the provider.
pub fn set_global_operation_logger(logger: Arc<dyn OperationLogger>) {
    let _ = GLOBAL_LOGGER.set(logger);
}

/// Forwards domain macro calls to the global logger (registered with `set_log_fn`).
pub fn forward(
    level: LogLevel,
    context: &str,
    message: &str,
    detail: Option<&dyn std::fmt::Display>,
) {
    dispatch(level, context, message, detail);
}

/// Logs via the global logger at the given level, or fallback (message only).
fn dispatch(level: LogLevel, context: &str, message: &str, detail: Option<&dyn std::fmt::Display>) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        logger.log(level, context, message, detail);
    } else {
        match level {
            LogLevel::Error => tracing::error!(context = %context, "{}", message),
            LogLevel::Warn => tracing::warn!(context = %context, "{}", message),
            LogLevel::Info => tracing::info!(context = %context, "{}", message),
            LogLevel::Debug => tracing::debug!(context = %context, "{}", message),
            LogLevel::Trace => tracing::trace!(context = %context, "{}", message),
        }
    }
}
