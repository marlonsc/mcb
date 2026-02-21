//!
//! Logging facade: single registration point for the operation logger.
//!
//! Infra (or tests) call `set_log_fn` at startup so that macros `trace!` .. `error!`
//! forward to the real implementation. Domain has no dependency on tracing or infra.

use std::sync::OnceLock;

use crate::ports::LogLevel;

/// Function type that the infrastructure layer registers to handle log events.
pub type LogFn = fn(LogLevel, &str, &str, Option<&dyn std::fmt::Display>);

static LOG_FN: OnceLock<LogFn> = OnceLock::new();

/// Registers the log implementation (e.g. infra shim). Call once at startup.
pub fn set_log_fn(f: LogFn) {
    let _ = LOG_FN.set(f);
}

/// Dispatches to the registered log function, or no-op if none set.
#[inline]
pub fn log_operation(
    level: LogLevel,
    context: &str,
    message: &str,
    detail: Option<&dyn std::fmt::Display>,
) {
    if let Some(f) = LOG_FN.get() {
        f(level, context, message, detail);
    }
}
