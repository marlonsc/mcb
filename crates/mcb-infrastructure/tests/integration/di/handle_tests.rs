//! Handle<T> Tests
//!
//! Tests for the generic runtime-swappable provider handle.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mcb_infrastructure::di::Handle;

/// Test provider trait for unit tests
trait TestProvider: Send + Sync {
    fn value(&self) -> usize;
}

/// Simple test provider implementation
struct SimpleProvider {
    val: usize,
}

impl TestProvider for SimpleProvider {
    fn value(&self) -> usize {
        self.val
    }
}

#[test]
fn test_handle_get_returns_initial_provider() {
    let provider = Arc::new(SimpleProvider { val: 42 });
    let handle: Handle<dyn TestProvider> = Handle::new(provider);

    assert_eq!(handle.get().value(), 42);
}

#[test]
fn test_handle_set_updates_provider() {
    let provider1 = Arc::new(SimpleProvider { val: 1 });
    let handle: Handle<dyn TestProvider> = Handle::new(provider1);

    assert_eq!(handle.get().value(), 1);

    let provider2 = Arc::new(SimpleProvider { val: 2 });
    handle.set(provider2);

    assert_eq!(handle.get().value(), 2);
}

#[test]
fn test_handle_get_returns_cloned_arc() {
    let provider = Arc::new(SimpleProvider { val: 100 });
    let handle: Handle<dyn TestProvider> = Handle::new(provider);

    let p1 = handle.get();
    let p2 = handle.get();

    // Both should point to the same underlying data
    assert_eq!(p1.value(), p2.value());
}

#[test]
fn test_handle_concurrent_access() {
    use std::thread;

    let provider = Arc::new(SimpleProvider { val: 0 });
    let handle = Arc::new(Handle::<dyn TestProvider>::new(provider));

    let counter = Arc::new(AtomicUsize::new(0));

    let threads: Vec<_> = (0..10)
        .map(|_| {
            let handle = Arc::clone(&handle);
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..100 {
                    let _ = handle.get();
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();

    for t in threads {
        t.join().expect("Thread panicked");
    }

    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}

#[test]
fn test_handle_debug_impl() {
    let provider = Arc::new(SimpleProvider { val: 1 });
    let handle: Handle<dyn TestProvider> = Handle::new(provider);

    let debug_str = format!("{handle:?}");
    assert!(debug_str.contains("Handle"));
}
