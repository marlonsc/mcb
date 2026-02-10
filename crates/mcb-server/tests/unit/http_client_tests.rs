use mcb_server::transport::http_client::HttpClientTransport;
use std::time::Duration;

#[test]
fn test_http_client_creation() {
    let client = HttpClientTransport::new(
        "http://localhost:18080".to_string(),
        Some("test".to_string()),
        Duration::from_secs(30),
    );
    assert!(client.is_ok());
}
