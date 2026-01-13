//! Collection utilities - demonstrates DRY principle

use std::collections::HashMap;

/// Collection utilities - demonstrates DRY principle
pub struct CollectionUtils;

impl CollectionUtils {
    /// Safe get with default - eliminates repetitive unwrap_or patterns
    pub fn get_or_default<T: Clone>(map: &HashMap<String, T>, key: &str, default: T) -> T {
        map.get(key).cloned().unwrap_or(default)
    }

    /// Safe get with custom default function - more flexible
    pub fn get_or_else<T: Clone, F>(map: &HashMap<String, T>, key: &str, default_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        map.get(key).cloned().unwrap_or_else(default_fn)
    }

    /// Check if collection is empty - extracted common check
    pub fn is_empty<T>(collection: &[T]) -> bool {
        collection.is_empty()
    }

    /// Safe indexing with bounds check - prevents panics
    pub fn get_safe<T: Clone>(slice: &[T], index: usize) -> Option<T> {
        slice.get(index).cloned()
    }
}
