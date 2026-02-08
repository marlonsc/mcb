//! Test crate with intentional quality, KISS, and pattern violations.
//!
//! This crate serves as a realistic target for multiple validators,
//! containing code that looks plausible but has subtle problems.

use std::sync::{Arc, Mutex};

// ───────────────────────────────────────────────────
// Quality violations: unwrap, expect, TODO, FIXME
// ───────────────────────────────────────────────────

/// Processes configuration from a file path.
///
/// BUG(Quality): Contains .unwrap() in production code.
/// BUG(Quality): Contains TODO comment.
pub fn load_config(path: &str) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    // TODO: Add validation for config schema
    serde_json::from_str(&content).expect("invalid config JSON")
}

/// Merges two config values.
///
/// BUG(Quality): FIXME comment indicates known issue.
pub fn merge_configs(base: serde_json::Value, overlay: serde_json::Value) -> serde_json::Value {
    // FIXME: This doesn't handle nested merges correctly
    let mut result = base;
    if let (Some(base_obj), Some(overlay_obj)) = (result.as_object_mut(), overlay.as_object()) {
        for (key, value) in overlay_obj {
            base_obj.insert(key.clone(), value.clone());
        }
    }
    result
}

/// BUG(Quality): panic!() in production code — should use Result or error type.
pub fn validate_critical_config(value: &serde_json::Value) {
    if !value.is_object() {
        panic!("Configuration must be a JSON object");
    }
}

/// BUG(Quality): #[allow(dead_code)] hiding unused code.
#[allow(dead_code)]
struct InternalCache {
    entries: Vec<String>,
    max_size: usize,
}

#[allow(dead_code)]
fn unused_helper() -> bool {
    false
}

// ───────────────────────────────────────────────────
// KISS violations: too many fields, too many params
// ───────────────────────────────────────────────────

/// BUG(KISS): Too many struct fields (8) — should be decomposed.
///
/// A realistic example: this looks like a valid server parameters struct,
/// but it has grown beyond what's maintainable.
pub struct ServerParameters {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_ms: u64,
    pub tls_enabled: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub log_level: String,
}

/// BUG(KISS): Function with too many parameters (> 5).
///
/// Realistic scenario: initialization function that keeps accumulating args.
pub fn initialize_server(
    host: &str,
    port: u16,
    max_conn: u32,
    timeout: u64,
    tls: bool,
    log_level: &str,
) -> Result<(), String> {
    println!(
        "Starting server on {}:{} (max_conn={}, timeout={}ms, tls={}, log={})",
        host, port, max_conn, timeout, tls, log_level
    );
    Ok(())
}

// ───────────────────────────────────────────────────
// Async / pattern violations
// ───────────────────────────────────────────────────

/// BUG(AsyncPatterns): Blocking I/O calls inside async function.
///
/// Common antipattern: developer writes sync code in async fn without
/// thinking about blocking the tokio runtime's thread pool.
pub async fn async_file_processor(path: &str) -> Result<String, std::io::Error> {
    // BUG: std::fs::read_to_string is blocking
    let content = std::fs::read_to_string(path)?;

    // BUG: std::thread::sleep blocks the async runtime
    std::thread::sleep(std::time::Duration::from_millis(50));

    Ok(content.to_uppercase())
}

/// BUG(Patterns): `Arc<Mutex<>>` in async code — should use `tokio::sync::Mutex`.
///
/// This is a very common mistake when porting sync code to async.
pub async fn shared_state_handler(data: Arc<Mutex<Vec<String>>>, new_item: String) {
    let mut guard = data.lock().unwrap(); // BUG: std Mutex in async
    guard.push(new_item);
}

/// Proper async handler (clean — no violations).
pub async fn async_proper_handler() -> Result<String, std::io::Error> {
    let content = tokio::fs::read_to_string("example.txt").await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(content)
}

// ───────────────────────────────────────────────────
// Documentation violations
// ───────────────────────────────────────────────────

/// BUG(Documentation): Public struct without field documentation.
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

pub struct UndocumentedStruct {
    pub field1: String,
    pub field2: i32,
}

pub fn undocumented_function() -> bool {
    true
}

// ───────────────────────────────────────────────────
// Performance violations (PERF001–PERF005)
// ───────────────────────────────────────────────────

/// Helper that accepts a cloned value (used to trigger PERF001).
fn consume_template(_t: serde_json::Value) {}

/// BUG(PERF001): Clone in loop — .clone() in function argument, NOT let binding.
pub fn batch_process(items: Vec<String>, template: serde_json::Value) {
    for _item in &items {
        consume_template(template.clone()); // PERF001: clone in function arg
    }
}

/// BUG(PERF002): Allocation in loop — Vec::new() inside for loop.
pub fn allocate_in_loop(count: usize) {
    for _ in 0..count {
        let _buf: Vec<u8> = Vec::new(); // PERF002: allocation in loop
        println!("allocated");
    }
}

/// BUG(PERF003): Arc<Mutex> overuse — Mutex<bool> should be AtomicBool.
pub struct OveruseExample {
    pub flag: std::sync::Mutex<bool>, // PERF003: Mutex<bool>
}

/// BUG(PERF004): Inefficient iterator — .iter().cloned().take().
pub fn inefficient_iter(data: &[String]) -> Vec<String> {
    data.iter().cloned().take(5).collect() // PERF004
}

/// BUG(PERF005): Inefficient string — format!("{}", var) instead of var.to_string().
pub fn inefficient_string(name: &str) -> String {
    format!("{}", name) // PERF005
}

// ───────────────────────────────────────────────────
// Pattern violations (PAT001, PAT004)
// ───────────────────────────────────────────────────

/// Concrete service implementation (used to trigger PAT001).
pub struct CacheServiceImpl;

/// BUG(PAT001): Concrete type in DI — Arc<CacheServiceImpl> instead of Arc<dyn CacheService>.
pub struct AppContainer {
    pub cache: Arc<CacheServiceImpl>, // PAT001: concrete DI
}

/// BUG(PAT004): Raw result type — std::result::Result instead of crate::Result.
pub fn raw_result_usage() -> std::result::Result<(), String> {
    // PAT004
    Ok(())
}

// ───────────────────────────────────────────────────
// Organization violations: magic numbers
// ───────────────────────────────────────────────────

/// BUG(Organization): Magic numbers in business logic.
pub fn calculate_pricing(base_price: f64, quantity: u32) -> f64 {
    let subtotal = base_price * quantity as f64;
    let tax = subtotal * 0.0875; // BUG: magic number for tax rate
    let shipping = if quantity > 10 { 0.0 } else { 15.99 }; // BUG: magic numbers

    // BUG: magic number — 5-digit threshold for special discount
    if subtotal > 10000.0 {
        subtotal * 0.95 + tax + shipping // BUG: magic discount rate
    } else {
        subtotal + tax + shipping
    }
}

// ───────────────────────────────────────────────────
// Implementation quality violations
// ───────────────────────────────────────────────────

/// BUG(Implementation): Contains todo!() and unimplemented!() macros.
pub fn not_ready_yet() -> String {
    todo!("implement this properly")
}

/// BUG(Implementation): Empty method bodies (stubs).
pub struct EmptyService;

impl EmptyService {
    pub fn process(&self) -> Result<(), String> {
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// BUG(Implementation): Empty catch-all in match.
pub fn handle_message(msg_type: &str) {
    match msg_type {
        "create" => println!("Creating"),
        "update" => println!("Updating"),
        "delete" => println!("Deleting"),
        _ => {} // BUG: silently ignores unknown message types
    }
}

// ───────────────────────────────────────────────────
// Inline tests — tests_org violation
// ───────────────────────────────────────────────────

/// BUG(TestOrg): Inline test module — should be in separate test file.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_exists() {
        // This test just proves the function exists
        // BUG(TestOrg): test function doesn't start with 'test_' prefix... wait it does.
    }

    #[test]
    fn pricing_basic() {
        // BUG(TestOrg): Test name doesn't follow test_ prefix convention
        let price = calculate_pricing(100.0, 5);
        assert!(price > 0.0);
    }

    #[test]
    fn bad_name() {
        // BUG(TestOrg): Test name doesn't follow test_ convention
        assert!(true);
    }
}
