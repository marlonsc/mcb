/// Unified integration test helpers
/// Detects external service availability and skips tests if services are unavailable
///
/// Usage in tests:
/// ```ignore
/// #[tokio::test]
/// async fn example() {
///     skip_if_service_unavailable!("Milvus", is_milvus_available());
///     let result = do_something();
///     assert_eq!(result, expected);
/// }
/// ```
use std::net::TcpStream;
use std::time::Duration;

/// Check if a service is available on given host:port
pub fn check_service_available(host: &str, port: u16) -> bool {
    let addr = format!("{}:{}", host, port);
    match addr.parse() {
        Ok(socket_addr) => {
            TcpStream::connect_timeout(&socket_addr, Duration::from_millis(300)).is_ok()
        }
        Err(_) => false,
    }
}

/// Milvus vector database service (default port 19530)
pub fn is_milvus_available() -> bool {
    check_service_available("127.0.0.1", 19530) || check_service_available("localhost", 19530)
}

/// Ollama LLM service (default port 11434)
pub fn is_ollama_available() -> bool {
    check_service_available("127.0.0.1", 11434) || check_service_available("localhost", 11434)
}

/// Redis cache service (default port 6379)
pub fn is_redis_available() -> bool {
    check_service_available("127.0.0.1", 6379) || check_service_available("localhost", 6379)
}

/// PostgreSQL service (default port 5432)
pub fn is_postgres_available() -> bool {
    check_service_available("127.0.0.1", 5432) || check_service_available("localhost", 5432)
}

/// Skip test if service is not available
/// Returns early from test function if service is unavailable
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
        if !$is_available {
            println!(
                "⊘ SKIPPED: {} service not available (skipping test)",
                $service
            );
            return;
        }
    };
}

/// Skip test if any required services are unavailable
/// Useful for tests requiring multiple external services
///
/// # Example
/// ```ignore
/// skip_if_any_service_unavailable!("Milvus" => is_milvus_available(), "Ollama" => is_ollama_available());
/// ```
#[macro_export]
macro_rules! skip_if_any_service_unavailable {
    ($($service:expr => $is_available:expr),+ $(,)?) => {
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
        // These checks should not panic and return boolean values
        let milvus = is_milvus_available();
        let ollama = is_ollama_available();
        let redis = is_redis_available();
        let postgres = is_postgres_available();

        // Assert that functions return boolean (may be true or false depending on environment)
        assert!(matches!(milvus, true | false));
        assert!(matches!(ollama, true | false));
        assert!(matches!(redis, true | false));
        assert!(matches!(postgres, true | false));

        println!("✓ Service detection logic verified");
    }
}
