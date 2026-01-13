//! JSON Value Extension - Replaces .get().and_then(|v| v.as_*()) pattern (DRY)
//!
//! Provides convenient accessor methods for JSON values with default fallbacks

use std::collections::HashMap;

/// Extension trait for serde_json::Value with convenient accessor methods
///
/// Replaces the verbose pattern:
/// ```ignore
/// meta.get("key").and_then(|v| v.as_str()).unwrap_or("default")
/// ```
/// With:
/// ```ignore
/// meta.str_or("key", "default")
/// ```
pub trait JsonExt {
    /// Get string value or default (replaces .get().and_then(as_str).unwrap_or)
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str;

    /// Get owned string value or default
    fn string_or(&self, key: &str, default: &str) -> String;

    /// Get i64 value or default
    fn i64_or(&self, key: &str, default: i64) -> i64;

    /// Get u64 value or default
    fn u64_or(&self, key: &str, default: u64) -> u64;

    /// Get f64 value or default
    fn f64_or(&self, key: &str, default: f64) -> f64;

    /// Get bool value or default
    fn bool_or(&self, key: &str, default: bool) -> bool;

    /// Get optional string (replaces .get().and_then(as_str))
    fn opt_str(&self, key: &str) -> Option<&str>;

    /// Get optional i64
    fn opt_i64(&self, key: &str) -> Option<i64>;

    /// Get optional u64
    fn opt_u64(&self, key: &str) -> Option<u64>;
}

impl JsonExt for serde_json::Value {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}

/// Extension trait for HashMap<String, serde_json::Value>
impl JsonExt for HashMap<String, serde_json::Value> {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}

/// Extension trait for serde_json::Map<String, Value>
impl JsonExt for serde_json::Map<String, serde_json::Value> {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}
