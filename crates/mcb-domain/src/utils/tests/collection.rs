//! Unique collection name generator for test isolation.
//!
//! Ensures each test uses a unique collection name to avoid interference.

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique collection name with prefix for test isolation.
pub fn unique_collection(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tid = std::thread::current().id().as_u64();
    format!("test_{prefix}_{id}_{tid}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_collection_format_has_prefix() {
        let name = unique_collection("mypfx");
        assert!(name.starts_with("test_mypfx_"), "unexpected format: {name}");
    }

    #[test]
    fn unique_collection_names_differ_between_calls() {
        let a = unique_collection("col");
        let b = unique_collection("col");
        assert_ne!(a, b, "successive calls must return distinct names");
    }
}
