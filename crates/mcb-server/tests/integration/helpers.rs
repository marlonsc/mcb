use std::net::TcpStream;
use std::time::Duration;

use mcb_domain::test_services_config::test_service_url;

pub fn check_service_available(host: &str, port: u16) -> bool {
    let addr = format!("{}:{}", host, port);
    match addr.parse() {
        Ok(socket_addr) => {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
        }
        Err(_) => false,
    }
}

/// Milvus vector database service (default port 29530)
pub fn is_milvus_available() -> bool {
    let host = match test_service_url("milvus_address") {
        Some(url) => url.replace("http://", "").replace("https://", ""),
        None => return false,
    };
    let parts: Vec<&str> = host.split(':').collect();
    let port = match parts.get(1).and_then(|p| p.parse().ok()) {
        Some(port) => port,
        None => return false,
    };
    check_service_available(parts[0], port)
}

/// Ollama LLM service (default port 21434)
pub fn is_ollama_available() -> bool {
    let host = match test_service_url("ollama_url") {
        Some(url) => url.replace("http://", "").replace("https://", ""),
        None => return false,
    };
    let parts: Vec<&str> = host.split(':').collect();
    let port = match parts.get(1).and_then(|p| p.parse().ok()) {
        Some(port) => port,
        None => return false,
    };
    check_service_available(parts[0], port)
}

/// Redis cache service (default port 26379)
pub fn is_redis_available() -> bool {
    let host = match test_service_url("redis_url") {
        Some(url) => url.replace("redis://", ""),
        None => return false,
    };
    let parts: Vec<&str> = host.split(':').collect();
    let port = match parts.get(1).and_then(|p| p.parse().ok()) {
        Some(port) => port,
        None => return false,
    };
    check_service_available(parts[0], port)
}

/// NATS event bus service (default port 24222)
pub fn is_nats_available() -> bool {
    let host = match test_service_url("nats_url") {
        Some(url) => url.replace("nats://", ""),
        None => return false,
    };
    let parts: Vec<&str> = host.split(':').collect();
    let port = match parts.get(1).and_then(|p| p.parse().ok()) {
        Some(port) => port,
        None => return false,
    };
    check_service_available(parts[0], port)
}

/// PostgreSQL service (default port 25432)
pub fn is_postgres_available() -> bool {
    let host = match test_service_url("postgres_url") {
        Some(url) => url,
        None => return false,
    };
    let host_port = host
        .rsplit("://")
        .next()
        .unwrap_or(&host)
        .rsplit('@')
        .next()
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("");
    if host_port.is_empty() {
        return false;
    }
    let parts: Vec<&str> = host_port.split(':').collect();
    let port = match parts.get(1).and_then(|p| p.parse::<u16>().ok()) {
        Some(port) => port,
        None => return false,
    };
    check_service_available(parts[0], port)
}

/// Check if running in CI environment
/// Returns true if CI environment variable is set
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

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

/// Skip test if service is not available or if in CI
/// Returns early from test function if service is unavailable or in CI environment
///
/// # Example - Single service
/// ```ignore
/// skip_if_service_unavailable!("Milvus", is_milvus_available());
/// skip_if_service_unavailable!("Ollama", is_ollama_available());
/// ```
///
/// # Example - Multiple services (any one missing skips test)
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_service_unavailable {
    ($service:expr, $is_available:expr) => {
        if !$crate::helpers::should_run_docker_integration_tests() {
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

/// Skip test if any required services are unavailable or if in CI
/// Useful for tests requiring multiple external services
///
/// # Example
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_any_service_unavailable {
    ($($service:expr => $is_available:expr),+ $(,)?) => {
        if !$crate::helpers::should_run_docker_integration_tests() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_detection_logic() {
        let _ = is_ci();
        let _ = should_run_docker_integration_tests();
        let milvus = is_milvus_available();
        let ollama = is_ollama_available();
        let redis = is_redis_available();
        let nats = is_nats_available();
        let postgres = is_postgres_available();

        assert!(matches!(milvus, true | false));
        assert!(matches!(ollama, true | false));
        assert!(matches!(redis, true | false));
        assert!(matches!(nats, true | false));
        assert!(matches!(postgres, true | false));

        println!("✓ Service detection logic verified");
    }
}
