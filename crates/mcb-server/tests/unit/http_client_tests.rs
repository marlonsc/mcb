use mcb_server::transport::http_client::HttpClientTransport;
use std::time::Duration;

#[test]
fn test_http_client_creation() {
    let client = HttpClientTransport::new_with_session_source(
        "http://localhost:18080".to_owned(),
        Some("test".to_owned()),
        Duration::from_secs(30),
        None,
        None,
    );
    assert!(client.is_ok());
}
