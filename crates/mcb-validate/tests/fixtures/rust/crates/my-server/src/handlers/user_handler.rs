//! User HTTP handlers
//!
//! Handles REST API routes for user management. This module demonstrates
//! several common antipatterns in handler code that validators should detect.

use serde_json::json;
use std::sync::{Arc, Mutex};

/// Shared application state.
///
/// BUG(Patterns): Uses `Arc<Mutex<>>` in async context — should use
/// `tokio::sync::Mutex` or `RwLock` instead for async-safe locking.
pub struct AppState {
    pub users: Arc<Mutex<Vec<String>>>,
    pub config: Arc<Mutex<serde_json::Value>>,
}

/// Creates a new user.
///
/// BUG(ErrorBoundary): Uses bare `?` without `.context()` or `.map_err()`.
/// Handler errors should always include context for debugging.
///
/// BUG(ErrorBoundary): Leaks internal error details to API response using
/// `format!("{:?}", err)` — exposes stack traces and internal types.
pub async fn create_user(
    state: Arc<Mutex<Vec<String>>>,
    body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // BUG: bare ? without context
    let parsed: serde_json::Value = serde_json::from_str(body)?;

    let name = parsed["name"].as_str().ok_or("missing name")?;

    // BUG(AsyncPatterns): Blocking call in async function —
    // std::fs::read_to_string blocks the tokio runtime thread
    let config = std::fs::read_to_string("/etc/app/config.toml")?;

    // BUG(AsyncPatterns): std::thread::sleep in async context
    std::thread::sleep(std::time::Duration::from_millis(100));

    // BUG: Locking std::sync::Mutex in async code can deadlock
    let mut users = state.lock().unwrap();
    users.push(name.to_string());

    Ok(json!({"status": "created", "name": name}).to_string())
}

/// Gets user by ID.
///
/// BUG(ErrorBoundary): Leaks internal error details in response body.
/// Never expose `Debug` formatting or `.to_string()` of internal errors.
pub async fn get_user(user_id: &str) -> Result<String, String> {
    if user_id.is_empty() {
        return Err("user_id is required".to_string());
    }

    // Simulate DB lookup that might fail
    let result: Result<String, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "user not found in database",
    ));

    match result {
        Ok(user) => Ok(user),
        // BUG: Leaks internal error via Debug formatting
        Err(e) => Err(format!("Internal error: {:?}", e)),
    }
}

/// Lists all users with pagination.
///
/// BUG(KISS): Too many parameters for a handler function.
pub async fn list_users(
    page: usize,
    per_page: usize,
    sort_by: &str,
    sort_order: &str,
    filter_role: Option<&str>,
) -> Result<String, String> {
    // BUG(Organization): Magic numbers scattered in handler logic
    let max_per_page = 100;
    let default_timeout = 30000;

    if per_page > max_per_page {
        return Err("per_page too large".to_string());
    }

    // BUG(Quality): Using .expect() in handler code — will panic on failure
    let timeout_str =
        std::env::var("REQUEST_TIMEOUT").unwrap_or_else(|_| default_timeout.to_string());
    let _timeout: u64 = timeout_str.parse().expect("invalid timeout");

    Ok(json!({
        "page": page,
        "per_page": per_page,
        "users": []
    })
    .to_string())
}

/// Deletes user — demonstrates missing error context pattern.
///
/// BUG(ErrorBoundary): Multiple bare `?` operators without .context().
pub async fn delete_user(user_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    // BUG: bare ? — no context on what we're reading
    let config = std::fs::read_to_string("config.toml")?;

    // BUG: bare ? — no context on what we're parsing
    let parsed: serde_json::Value = serde_json::from_str(&config)?;

    // BUG: bare ? — no context on what API we're calling
    let _response = reqwest::get(format!("https://api.example.com/users/{}", user_id)).await?;

    Ok(json!({"deleted": user_id}).to_string())
}

/// Health check handler — looks innocent but has hidden problems.
///
/// BUG(AsyncPatterns): Performs blocking I/O in async handler.
pub async fn health_check() -> Result<String, String> {
    // BUG: blocking DNS lookup in async
    let _addr = std::net::ToSocketAddrs::to_socket_addrs(&mut "db.example.com:5432")
        .map_err(|e| e.to_string())?;

    // BUG: blocking file read in async
    let _ =
        std::fs::metadata("/var/run/app.pid").map_err(|e| format!("Health check failed: {}", e))?;

    Ok(json!({"status": "healthy"}).to_string())
}

/// Internal error handler that leaks details.
///
/// BUG(ErrorBoundary): Exposes internal error information to client.
pub fn error_response(err: &dyn std::error::Error) -> String {
    // BAD: Exposes full error chain to API consumers
    format!(
        "Error: {}. Debug: {:?}. Source: {:?}",
        err,
        err,
        err.source()
    )
}

/// BUG(Refactoring): Duplicate type — same name as in my-domain crate.
pub struct User {
    pub id: String,
    pub name: String,
}
