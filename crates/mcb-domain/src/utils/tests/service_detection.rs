//! External service detection and CI environment helpers.
//!
//! Centralized in `mcb-domain` so every crate can detect service availability
//! and skip tests when external dependencies are unavailable.

use std::net::TcpStream;
use std::time::Duration;

/// Check if a service is available by attempting a TCP connection.
#[must_use]
pub fn check_service_available(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    match addr.parse() {
        Ok(socket_addr) => {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
        }
        Err(_) => false,
    }
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
    is_service_available_from_config("milvus_address")
}

/// Ollama LLM service.
#[must_use]
pub fn is_ollama_available() -> bool {
    is_service_available_from_config("ollama_url")
}

/// Redis cache service.
#[must_use]
pub fn is_redis_available() -> bool {
    is_service_available_from_config("redis_url")
}

/// PostgreSQL service.
#[must_use]
pub fn is_postgres_available() -> bool {
    is_service_available_from_config("postgres_url")
}

/// Check if running in CI environment.
#[must_use]
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Check if Docker integration tests should run.
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

/// Skip test if service is not available or if in CI.
///
/// # Example
/// ```ignore
/// skip_if_service_unavailable!("Milvus", is_milvus_available());
/// ```
#[macro_export]
macro_rules! skip_if_service_unavailable {
    ($service:expr, $is_available:expr) => {
        if !$crate::utils::tests::service_detection::should_run_docker_integration_tests() {
            println!("⊘ SKIPPED: Docker integration tests disabled in this environment");
            return;
        }
        if !$is_available {
            println!(
                "⊘ SKIPPED: {} service not available (skipping test)",
                $service
            );
            return;
        }
    };
}

/// Skip test if any required services are unavailable.
///
/// # Example
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_any_service_unavailable {
    ($($service:expr => $is_available:expr),+ $(,)?) => {
        if !$crate::utils::tests::service_detection::should_run_docker_integration_tests() {
            println!("⊘ SKIPPED: Docker integration tests disabled in this environment");
            return;
        }
        $(
            if !$is_available {
                println!(
                    "⊘ SKIPPED: {} service not available (skipping test)",
                    $service
                );
                return;
            }
        )+
    };
}
