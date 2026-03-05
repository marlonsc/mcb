//! Logging facade: single registration point for the operation logger.
//!
//! Provides `set_log_fn` + `dispatch` (OnceLock-based), plus a built-in
//! implementation: `stderr_log_fn` (pure std).

use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::ports::LogLevel;

/// Function type that the infrastructure layer registers to handle log events.
pub type LogFn = fn(LogLevel, &str, &str, Option<&dyn std::fmt::Display>);

static LOG_FN: OnceLock<LogFn> = OnceLock::new();
static STDERR_LOG_LEVEL: AtomicU8 = AtomicU8::new(2);

/// Registers the log implementation (e.g. infra shim). Call once at startup.
pub fn set_log_fn(f: LogFn) {
    let _ = LOG_FN.set(f);
}

/// Dispatches to the registered log function, or no-op if none set.
#[inline]
pub fn dispatch(
    level: LogLevel,
    context: &str,
    message: &str,
    detail: Option<&dyn std::fmt::Display>,
) {
    if let Some(f) = LOG_FN.get() {
        f(level, context, message, detail);
    }
}

/// Sets the minimum log level for stderr output.
pub fn set_stderr_log_level(level: LogLevel) {
    let mapped = match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
        LogLevel::Trace => 4,
    };
    STDERR_LOG_LEVEL.store(mapped, Ordering::Relaxed);
}

fn level_to_u8(level: LogLevel) -> u8 {
    match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
        LogLevel::Trace => 4,
    }
}

/// A log function that filters by level and writes to stderr.
pub fn stderr_log_fn(
    level: LogLevel,
    context: &str,
    message: &str,
    detail: Option<&dyn std::fmt::Display>,
) {
    if level_to_u8(level) > STDERR_LOG_LEVEL.load(Ordering::Relaxed) {
        return;
    }

    let tag = match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => " WARN",
        LogLevel::Info => " INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    };

    if let Some(d) = detail {
        let _ = writeln!(std::io::stderr(), "[{tag}] {context}: {message} ({d})");
    } else {
        let _ = writeln!(std::io::stderr(), "[{tag}] {context}: {message}");
    }
}
