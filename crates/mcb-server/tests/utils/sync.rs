//! Shared test helpers to reduce code duplication

use rstest::rstest;
use std::sync::{Arc, Mutex};

/// Creates an Arc<Mutex<T>> with the given initial value
///
/// This helper reduces boilerplate in test mocks where we frequently need
/// thread-safe mutable state.
pub fn arc_mutex<T>(value: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(value))
}

/// Creates an Arc<Mutex<Vec<T>>> with an empty vector
pub fn arc_mutex_vec<T>() -> Arc<Mutex<Vec<T>>> {
    arc_mutex(Vec::new())
}

/// Creates an Arc<Mutex<`HashMap`<K, V>>> with an empty hashmap
pub fn arc_mutex_hashmap<K, V>() -> Arc<Mutex<std::collections::HashMap<K, V>>>
where
    K: Eq + std::hash::Hash,
{
    arc_mutex(std::collections::HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[test]
    fn test_arc_mutex() {
        let value = arc_mutex(42);
        assert_eq!(*value.lock().unwrap(), 42);
    }

    #[rstest]
    #[test]
    fn test_arc_mutex_vec() {
        let vec: Arc<Mutex<Vec<String>>> = arc_mutex_vec();
        assert!(vec.lock().unwrap().is_empty());
    }

    #[rstest]
    #[test]
    fn test_arc_mutex_hashmap() {
        let map: Arc<Mutex<std::collections::HashMap<String, i32>>> = arc_mutex_hashmap();
        assert!(map.lock().unwrap().is_empty());
    }
}
