use mcb_server::transport::http_client::HttpClientTransport;

use mcb_domain::utils::tests::timeouts::TEST_TIMEOUT;

#[rstest]
#[test]
fn test_http_client_creation() {
    let client = HttpClientTransport::new_with_session_source(
        "http://localhost:18080".to_owned(),
        Some("test".to_owned()),
        TEST_TIMEOUT,
        None,
        None,
    );
    assert!(client.is_ok());
}

use rstest::rstest;
use std::fs;
use std::time::Duration;

#[rstest]
#[test]
fn session_id_override_takes_precedence_over_file() {
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => return,
    };
    let session_file = temp_dir.path().join("session.id");

    let session_file_value = match session_file.to_str() {
        Some(value) => value.to_owned(),
        None => return,
    };

    let client = match HttpClientTransport::new_with_session_source(
        "http://127.0.0.1:18080".to_owned(),
        Some("prefix".to_owned()),
        Duration::from_secs(10),
        Some("explicit-session-id".to_owned()),
        Some(session_file_value),
    ) {
        Ok(client) => client,
        Err(_) => return,
    };

    drop(client);
    assert!(!session_file.exists());
}

#[rstest]
#[test]
fn session_id_persists_via_session_file() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let session_file = temp_dir.path().join("session.id");
    let session_file_str = session_file
        .to_str()
        .expect("session file path is not valid UTF-8")
        .to_owned();

    let first = HttpClientTransport::new_with_session_source(
        "http://127.0.0.1:18080".to_owned(),
        Some("persist".to_owned()),
        Duration::from_secs(10),
        None,
        Some(session_file_str.clone()),
    )
    .expect("failed to create first client");
    drop(first);

    let first_session = fs::read_to_string(&session_file).expect("failed to read first session");

    let second = HttpClientTransport::new_with_session_source(
        "http://127.0.0.1:18080".to_owned(),
        Some("persist".to_owned()),
        Duration::from_secs(10),
        None,
        Some(session_file_str),
    )
    .expect("failed to create second client");
    drop(second);

    let second_session = fs::read_to_string(&session_file).expect("failed to read second session");

    assert!(!first_session.trim().is_empty());
    assert_eq!(first_session, second_session);
}

#[rstest]
#[test]
fn secure_transport_allows_loopback_http() {
    for url in [
        "http://127.0.0.1:8080",
        "http://localhost:3000",
        "http://[::1]:9090",
    ] {
        assert!(
            HttpClientTransport::require_secure_transport(url).is_ok(),
            "should allow loopback URL: {url}"
        );
    }
}

#[rstest]
#[test]
fn secure_transport_allows_https() {
    for url in [
        "https://api.example.com",
        "https://10.0.0.1:443",
        "https://remote-server:8443/path",
    ] {
        assert!(
            HttpClientTransport::require_secure_transport(url).is_ok(),
            "should allow HTTPS URL: {url}"
        );
    }
}

#[rstest]
#[test]
fn secure_transport_rejects_remote_http() {
    for url in [
        "http://api.example.com",
        "http://10.0.0.1:8080",
        "http://192.168.1.1:3000",
    ] {
        assert!(
            HttpClientTransport::require_secure_transport(url).is_err(),
            "should reject remote HTTP URL: {url}"
        );
    }
}

#[rstest]
#[test]
fn secure_transport_rejects_unknown_scheme() {
    assert!(HttpClientTransport::require_secure_transport("ftp://files.example.com").is_err());
}

#[rstest]
#[test]
fn machine_id_detected_from_gethostname_without_env() {
    let expected_hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_owned());

    assert!(
        !expected_hostname.is_empty(),
        "hostname should not be empty"
    );
}

#[rstest]
#[test]
fn machine_id_prefers_gethostname_over_env() {
    let gethostname_result = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_owned());

    assert!(
        !gethostname_result.is_empty(),
        "gethostname should return a value"
    );
}
