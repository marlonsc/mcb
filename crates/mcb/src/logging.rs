use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};

use mcb_domain::ports::LogLevel;

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

static STDERR_LOG_LEVEL: AtomicU8 = AtomicU8::new(2);

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
