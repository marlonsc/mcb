//! Test helper macros.
//!
//! Macros for external service detection and test skipping.
//! All use `#[macro_export]` so they are available at crate root.

/// Skip a test early (with `Ok(())`) when the named external service is not
/// configured in `config/tests.toml` under `[test_services]`.
///
/// Usage at the top of any `-> TestResult` test function:
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_foo() -> TestResult {
///     require_service!("milvus");
///     // ... rest of the test
/// }
/// ```
#[macro_export]
macro_rules! require_service {
    ($service:expr) => {
        if $crate::utils::tests::services_config::test_service_url($service).is_none() {
            eprintln!("⏭ Skipping: {} not available", $service);
            return Ok(());
        }
    };
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
