//!
//! Logging macros: same style as `tracing::(trace|debug|info|warn|error)!`,
//! but dispatch via the domain log facade so infra can plug the real logger.

/// Logs at trace level via the domain log facade.
#[macro_export]
macro_rules! trace {
    ($ctx:expr, $msg:expr) => {
        $crate::infra::logging::dispatch(
            $crate::ports::LogLevel::Trace,
            $ctx,
            $msg,
            None::<&dyn std::fmt::Display>,
        )
    };
    ($ctx:expr, $msg:expr, $detail:expr) => {
        $crate::infra::logging::dispatch($crate::ports::LogLevel::Trace, $ctx, $msg, Some($detail))
    };
}

/// Logs at debug level via the domain log facade.
#[macro_export]
macro_rules! debug {
    ($ctx:expr, $msg:expr) => {
        $crate::infra::logging::dispatch(
            $crate::ports::LogLevel::Debug,
            $ctx,
            $msg,
            None::<&dyn std::fmt::Display>,
        )
    };
    ($ctx:expr, $msg:expr, $detail:expr) => {
        $crate::infra::logging::dispatch($crate::ports::LogLevel::Debug, $ctx, $msg, Some($detail))
    };
}

/// Logs at info level via the domain log facade.
#[macro_export]
macro_rules! info {
    ($ctx:expr, $msg:expr) => {
        $crate::infra::logging::dispatch(
            $crate::ports::LogLevel::Info,
            $ctx,
            $msg,
            None::<&dyn std::fmt::Display>,
        )
    };
    ($ctx:expr, $msg:expr, $detail:expr) => {
        $crate::infra::logging::dispatch($crate::ports::LogLevel::Info, $ctx, $msg, Some($detail))
    };
}

/// Logs at warn level via the domain log facade.
#[macro_export]
macro_rules! warn {
    ($ctx:expr, $msg:expr) => {
        $crate::infra::logging::dispatch(
            $crate::ports::LogLevel::Warn,
            $ctx,
            $msg,
            None::<&dyn std::fmt::Display>,
        )
    };
    ($ctx:expr, $msg:expr, $detail:expr) => {
        $crate::infra::logging::dispatch($crate::ports::LogLevel::Warn, $ctx, $msg, Some($detail))
    };
}

/// Logs at error level via the domain log facade.
#[macro_export]
macro_rules! error {
    ($ctx:expr, $msg:expr) => {
        $crate::infra::logging::dispatch(
            $crate::ports::LogLevel::Error,
            $ctx,
            $msg,
            None::<&dyn std::fmt::Display>,
        )
    };
    ($ctx:expr, $msg:expr, $detail:expr) => {
        $crate::infra::logging::dispatch($crate::ports::LogLevel::Error, $ctx, $msg, Some($detail))
    };
}
