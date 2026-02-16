use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn unique_collection(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tid = std::thread::current().id();
    format!("test_{prefix}_{id}_{tid:?}")
}

#[cfg(test)]
mod tests {
    use super::unique_collection;

    #[test]
    fn unique_collection_generates_prefixed_name() {
        let name = unique_collection("abc");
        assert!(name.starts_with("test_abc_"));
    }
}
