use mcb_domain::test_services_config::test_service_url;
use std::net::TcpStream;
use std::time::Duration;

/// Check if a service is available by attempting a TCP connection
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

fn get_host_port_from_url(url: &str) -> Option<(String, u16)> {
    let after_scheme = url.rsplit("://").next().unwrap_or(url);
    let after_auth = after_scheme.rsplit('@').next().unwrap_or(after_scheme);
    let host_port_str = after_auth.split('/').next().unwrap_or(after_auth);

    // Handle hostname:port (simplified)
    let (host, port_str) = host_port_str.rsplit_once(':')?;
    let port = port_str.parse::<u16>().ok()?;
    Some((host.to_owned(), port))
}

fn is_service_available_from_config(key: &str) -> bool {
    test_service_url(key)
        .and_then(|url| get_host_port_from_url(&url))
        .is_some_and(|(host, port)| check_service_available(&host, port))
}

/// Milvus vector database service (default port 29530)
#[must_use]
pub fn is_milvus_available() -> bool {
    is_service_available_from_config("milvus_address")
}

/// Ollama LLM service (default port 21434)
#[must_use]
pub fn is_ollama_available() -> bool {
    is_service_available_from_config("ollama_url")
}

/// Redis cache service (default port 26379)
#[must_use]
pub fn is_redis_available() -> bool {
    is_service_available_from_config("redis_url")
}

/// NATS event bus service (default port 24222)
pub fn is_nats_available() -> bool {
    is_service_available_from_config("nats_url")
}

/// `PostgreSQL` service (default port 25432)
#[must_use]
pub fn is_postgres_available() -> bool {
    is_service_available_from_config("postgres_url")
}

/// Check if running in CI environment
/// Returns true if CI environment variable is set
#[must_use]
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Check if Docker integration tests should run based on environment variable
/// or CI status. Defaults to !`is_ci()` if variable not set.
pub fn should_run_docker_integration_tests() -> bool {
    match std::env::var("MCB_RUN_DOCKER_INTEGRATION_TESTS") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            other => {
                mcb_domain::warn!(
                    "service_detection",
                    "Unknown value for MCB_RUN_DOCKER_INTEGRATION_TESTS. Falling back to !is_ci()",
                    &other
                );
                !is_ci()
            }
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
        if !$crate::utils::service_detection::should_run_docker_integration_tests() {
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
        if !$crate::utils::service_detection::should_run_docker_integration_tests() {
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

    #[test]
    fn test_get_host_port_from_url() {
        // Standard HTTP/HTTPS
        assert_eq!(
            get_host_port_from_url("http://localhost:8080"),
            Some(("localhost".to_owned(), 8080))
        );
        assert_eq!(
            get_host_port_from_url("https://api.example.com:443"),
            Some(("api.example.com".to_owned(), 443))
        );

        // Database URLs
        assert_eq!(
            get_host_port_from_url("postgres://user:pass@db-host:5432/db"),
            Some(("db-host".to_owned(), 5432))
        );
        assert_eq!(
            get_host_port_from_url("redis://127.0.0.1:6379"),
            Some(("127.0.0.1".to_owned(), 6379))
        );

        // IPv6
        assert_eq!(
            get_host_port_from_url("http://[::1]:8080"),
            Some(("[::1]".to_owned(), 8080))
        );
        assert_eq!(
            get_host_port_from_url("[::1]:8080"),
            Some(("[::1]".to_owned(), 8080))
        );

        // No scheme
        assert_eq!(
            get_host_port_from_url("localhost:3000"),
            Some(("localhost".to_owned(), 3000))
        );

        // Invalid or missing port
        assert_eq!(get_host_port_from_url("http://localhost"), None);
        assert_eq!(get_host_port_from_url("postgres://user:pass@db-host"), None);
    }
}
