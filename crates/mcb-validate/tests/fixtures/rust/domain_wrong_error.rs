// Domain service that improperly uses infrastructure error types.
// This file exercises `validate_layer_error_types()` — it should detect
// multiple distinct infra error types leaking into the domain layer.
//
// NOTE: This file must be placed under a `/domain/` path in the test crate
// structure, and must NOT be named `error.rs` (which is exempt).

use std::collections::HashMap;
use std::path::PathBuf;

/// Trait that defines how agents are persisted — this is correct,
/// pure domain interface with no infra dependencies.
pub trait AgentRepository: Send + Sync {
    fn find_by_id(&self, id: &str) -> Result<Option<Agent>, DomainError>;
    fn save(&self, agent: &Agent) -> Result<(), DomainError>;
}

/// Domain entity — should have NO infrastructure dependencies.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub tools: Vec<Tool>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
}

/// VIOLATION: Domain service directly uses std::io::Error as return type.
/// The domain layer should define its own error types, not depend on
/// infrastructure errors from std::io.
pub fn load_agent_config(path: &PathBuf) -> Result<Agent, std::io::Error> {
    let _content = std::fs::read_to_string(path)?;
    todo!("parse config")
}

/// VIOLATION: Uses reqwest::Error — an HTTP client library error
/// has no place in domain logic.
pub async fn validate_agent_endpoint(agent: &Agent) -> Result<bool, reqwest::Error> {
    let _url = format!("https://api.example.com/agents/{}", agent.id);
    // This function shouldn't know about HTTP at all
    Ok(true)
}

/// Correct: uses domain-specific error type
pub fn validate_agent_name(agent: &Agent) -> Result<(), DomainError> {
    if agent.name.is_empty() {
        return Err(DomainError::Validation("Agent name cannot be empty".into()));
    }
    Ok(())
}

/// VIOLATION: Uses sqlx::Error — database errors should not leak into domain.
/// The repository trait abstracts away persistence, so domain services
/// should never reference database-specific error types.
pub fn check_agent_exists(id: &str) -> Result<bool, sqlx::Error> {
    // Domain logic depending on a concrete database library
    let _ = id;
    Ok(false)
}

/// VIOLATION: Uses serde_json::Error in domain — serialization concerns
/// belong in the infrastructure/adapter layer, not in business rules.
pub fn parse_tool_definition(raw: &str) -> Result<Tool, serde_json::Error> {
    serde_json::from_str(raw)
}

/// VIOLATION: Uses tokio::sync::error — async runtime errors should not
/// appear in domain code, which should be runtime-agnostic.
pub async fn coordinate_agents(
    agents: &[Agent],
) -> Result<(), tokio::sync::broadcast::error::SendError<String>> {
    let _ = agents;
    Ok(())
}

/// Looks innocent but VIOLATION: has a hidden hyper::Error dependency
/// mixed into a seemingly legitimate function signature.
pub fn health_check() -> Result<(), hyper::Error> {
    // Domain shouldn't know about HTTP frameworks
    Ok(())
}

/// Correct domain error definition  — this is the proper pattern
#[derive(Debug)]
pub enum DomainError {
    NotFound(String),
    Validation(String),
    Conflict(String),
}
