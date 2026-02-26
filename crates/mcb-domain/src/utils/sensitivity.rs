//!
//! Redaction of sensitive values in Debug and Display.

use std::fmt;

/// Placeholder shown instead of sensitive data.
pub const REDACTED: &str = "REDACTED";

/// Wraps a value so that `Debug` and `Display` output `REDACTED`.
#[derive(Clone, Copy)]
pub struct Sensitive<T>(pub T);

impl<T> fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{REDACTED}")
    }
}

impl<T> fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{REDACTED}")
    }
}

