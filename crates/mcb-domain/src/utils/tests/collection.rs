//! Unique collection name generator for test isolation.
//!
//! Ensures each test uses a unique collection name to avoid interference.

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique collection name with prefix for test isolation.
pub fn unique_collection(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tid = std::thread::current().id();
    format!("test_{prefix}_{id}_{tid:?}")
}
