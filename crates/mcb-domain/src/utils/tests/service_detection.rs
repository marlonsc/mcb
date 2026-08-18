//! External service detection and CI environment helpers.
//!
//! Centralized in `mcb-domain` so every crate can detect service availability
//! and skip tests when external dependencies are unavailable.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Human-readable status of a configured external test service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    /// Config key in `config/tests.toml`.
    pub key: &'static str,
    /// Display name for reporting.
    pub name: &'static str,
    /// Whether the service answered the TCP probe.
    pub available: bool,
}

/// All external services that integration tests may depend on.
pub const EXTERNAL_SERVICES: &[(&str, &str)] = &[
    ("milvus", "Milvus"),
    ("ollama", "Ollama"),
    ("redis", "Redis"),
    ("postgres", "PostgreSQL"),
    ("nats", "NATS"),
    ("api_mock", "API Mock"),
];

/// Check if a service is available by attempting a TCP connection.
///
/// Supports both IP addresses and hostnames (e.g., "localhost").
#[must_use]
pub fn check_service_available(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    // Use ToSocketAddrs for DNS resolution (handles "localhost", etc.)
    let Ok(mut addrs) = addr.to_socket_addrs() else {
        return false;
    };
    addrs.any(|socket_addr| {
        TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
    })
}

/// Extract host and port from a URL string.
fn get_host_port_from_url(url: &str) -> Option<(String, u16)> {
    let after_scheme = url.rsplit("://").next().unwrap_or(url);
    let after_auth = after_scheme.rsplit('@').next().unwrap_or(after_scheme);
    let host_port_str = after_auth.split('/').next().unwrap_or(after_auth);
    let (host, port_str) = host_port_str.rsplit_once(':')?;
    let port = port_str.parse::<u16>().ok()?;
    Some((host.to_owned(), port))
}

/// Check if a service is available using `config/tests.toml` key.
fn is_service_available_from_config(key: &str) -> bool {
    crate::utils::tests::services_config::test_service_url(key)
        .and_then(|url| get_host_port_from_url(&url))
        .is_some_and(|(host, port)| check_service_available(&host, port))
}

/// Milvus vector database service.
#[must_use]
pub fn is_milvus_available() -> bool {
    is_service_available_from_config("milvus")
}

/// Ollama LLM service.
#[must_use]
pub fn is_ollama_available() -> bool {
    is_service_available_from_config("ollama")
}

/// Redis cache service.
#[must_use]
pub fn is_redis_available() -> bool {
    is_service_available_from_config("redis")
}

/// `PostgreSQL` service.
#[must_use]
pub fn is_postgres_available() -> bool {
    is_service_available_from_config("postgres")
}

/// NATS event bus service.
#[must_use]
pub fn is_nats_available() -> bool {
    is_service_available_from_config("nats")
}

/// API mock service.
#[must_use]
pub fn is_api_mock_available() -> bool {
    is_service_available_from_config("api_mock")
}

/// Return the availability status for every configured external service.
///
/// Services that are not configured in `config/tests.toml` are reported as
/// unavailable. The report is never cached so it reflects the current
/// environment.
#[must_use]
pub fn service_availability_summary() -> Vec<ServiceStatus> {
    EXTERNAL_SERVICES
        .iter()
        .map(|(key, name)| ServiceStatus {
            key,
            name,
            available: is_service_available_from_config(key),
        })
        .collect()
}

/// Format the service availability summary for console output.
#[must_use]
pub fn format_service_summary(summary: &[ServiceStatus]) -> String {
    let mut lines: Vec<String> = vec!["External service availability:".to_owned()];
    for status in summary {
        let marker = if status.available { "✓" } else { "✗" };
        lines.push(format!("  {} {} ({})", marker, status.name, status.key));
    }
    lines.join("\n")
}

/// Print the service availability summary to stdout.
#[allow(clippy::print_stdout)]
pub fn print_service_availability_summary() {
    println!(
        "{}",
        format_service_summary(&service_availability_summary())
    );
}

/// Check if every named service is both configured and reachable.
///
/// Useful for dynamic test groups that require a complete set of services.
#[must_use]
pub fn all_required_services_available(keys: &[&str]) -> bool {
    keys.iter().all(|key| is_service_available_from_config(key))
}

/// Check if running in CI environment.
#[must_use]
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Check if Docker integration tests should run.
#[must_use]
pub fn should_run_docker_integration_tests() -> bool {
    match std::env::var("MCB_RUN_DOCKER_INTEGRATION_TESTS") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            _ => !is_ci(),
        },
        Err(_) => !is_ci(),
    }
}

// `skip_if_service_unavailable!` and `skip_if_any_service_unavailable!` macros
// are defined in `crate::macros::testing` and available via `#[macro_export]`.
