// Handler module with multiple error handling violations.
// This file exercises `validate_error_context()` and `validate_leaked_errors()`.
//
// NOTE: This file must be placed under a `/handlers/` path in the test
// crate structure for the validators to scan it.

use std::collections::HashMap;

/// Request type for creating an agent
pub struct CreateAgentRequest {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
}

/// Response type (simplified)
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
}

impl ApiResponse {
    pub fn ok(body: String) -> Self {
        Self { status: 200, body }
    }

    pub fn error(status: u16, body: String) -> Self {
        Self { status, body }
    }
}

/// VIOLATION (ERR001): Multiple bare `?` operators without `.context()`.
/// Each `?` propagates errors without adding meaningful context about
/// what operation failed, making debugging in production very difficult.
pub async fn create_agent(
    req: CreateAgentRequest,
) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    // VIOLATION: bare ? — no context about what config file we tried to load
    let config = load_config("agents.toml")?;

    // VIOLATION: bare ? — no context about validation failure
    let validated = validate_request(&req)?;

    // Correct: uses .map_err() to add context
    let agent_id = generate_id().map_err(|e| format!("Failed to generate agent ID: {e}"))?;

    // VIOLATION: bare ? — if DB save fails, stack trace won't show which agent
    save_to_database(&agent_id, &validated)?;

    // VIOLATION: bare ? — network call with no context about target endpoint
    notify_webhook(&agent_id)?;

    Ok(ApiResponse::ok(format!("Created agent: {agent_id}")))
}

/// VIOLATION (ERR001): Handler with deeply nested error propagation.
/// The ? operators are hidden in a chain of operations making it
/// hard to trace which step failed.
pub async fn update_agent(
    id: &str,
    updates: HashMap<String, String>,
) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    // VIOLATION: bare ? — which database call failed?
    let mut agent = fetch_agent(id)?;

    for (key, value) in &updates {
        // Correct: properly contextualized error
        agent.insert(
            key.clone(),
            value
                .parse()
                .map_err(|e| format!("Invalid value for {key}: {e}"))?,
        );
    }

    // VIOLATION: bare ? — silent failure on persist
    persist_agent(id, &agent)?;

    // VIOLATION (ERR003): Debug formatting exposes internal error struct to API
    let audit_result = log_audit_event(id, "update");
    if let Err(e) = audit_result {
        return Ok(ApiResponse::error(500, format!("{:?}", e)));
    }

    Ok(ApiResponse::ok("Updated".into()))
}

/// VIOLATION (ERR003): Handler leaks internal error details in response body.
/// API consumers should see sanitized error messages, not internal stack traces.
pub async fn delete_agent(id: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    match remove_from_database(id) {
        Ok(()) => Ok(ApiResponse::ok("Deleted".into())),
        // VIOLATION: .to_string() on internal error exposed directly to client
        Err(e) => Ok(ApiResponse::error(500, e.to_string())),
    }
}

/// VIOLATION (ERR003): JSON response body contains formatted internal error.
/// This could expose database schema, file paths, or other sensitive info.
pub async fn get_agent_status(id: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let status = check_status(id);
    match status {
        Ok(s) => Ok(ApiResponse::ok(s)),
        Err(e) => {
            // VIOLATION: internal error details in JSON API response
            let body = serde_json::json!({ "error": format!("{:?}", e) });
            Ok(ApiResponse::error(500, body.to_string()))
        }
    }
}

/// Correct handler: proper error handling with context and sanitized responses.
/// This serves as the counter-example showing the right pattern.
pub async fn list_agents() -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let agents = fetch_all_agents().map_err(|e| format!("Failed to fetch agents list: {e}"))?;

    let serialized =
        serde_json::to_string(&agents).context("Failed to serialize agents response")?;

    Ok(ApiResponse::ok(serialized))
}

// ---- Stub functions (would be real service calls) ----
fn load_config(_path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    Ok(HashMap::new())
}
fn validate_request(
    _req: &CreateAgentRequest,
) -> Result<CreateAgentRequest, Box<dyn std::error::Error>> {
    todo!()
}
fn generate_id() -> Result<String, Box<dyn std::error::Error>> {
    Ok("agent-001".into())
}
fn save_to_database(
    _id: &str,
    _req: &CreateAgentRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
fn notify_webhook(_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
fn fetch_agent(_id: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    Ok(HashMap::new())
}
fn persist_agent(
    _id: &str,
    _data: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
fn log_audit_event(_id: &str, _action: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
fn remove_from_database(_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
fn check_status(_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok("healthy".into())
}
fn fetch_all_agents() -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    Ok(vec![])
}
