//! Thread-safe wrapper helpers for test state.
//!
//! Centralized in `mcb-domain` to reduce boilerplate in tests that need
//! `Arc<Mutex<T>>` containers.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// Creates an `Arc<Mutex<T>>` with the given initial value.
pub fn arc_mutex<T>(value: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(value))
}

/// Creates an `Arc<Mutex<Vec<T>>>` with an empty vector.
#[must_use]
pub fn arc_mutex_vec<T>() -> Arc<Mutex<Vec<T>>> {
    arc_mutex(Vec::new())
}

/// Creates an `Arc<Mutex<HashMap<K, V>>>` with an empty hashmap.
#[must_use]
pub fn arc_mutex_hashmap<K, V>() -> Arc<Mutex<HashMap<K, V>>>
where
    K: Eq + Hash,
{
    arc_mutex(HashMap::new())
}
