//! Service Lifecycle Handlers Tests
//!
//! Tests for the service lifecycle management HTTP handlers.

use mcb_server::admin::lifecycle_handlers::{ServiceInfoResponse, ServiceListResponse};

#[test]
fn test_service_list_response_serialization() {
    let response = ServiceListResponse {
        count: 2,
        services: vec![
            ServiceInfoResponse {
                name: "embedding".to_string(),
                state: "Running".to_string(),
            },
            ServiceInfoResponse {
                name: "vector_store".to_string(),
                state: "Stopped".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"count\":2"));
    assert!(json.contains("\"name\":\"embedding\""));
    assert!(json.contains("\"state\":\"Running\""));
}

#[test]
fn test_service_info_response_serialization() {
    let info = ServiceInfoResponse {
        name: "cache".to_string(),
        state: "Starting".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"name\":\"cache\""));
    assert!(json.contains("\"state\":\"Starting\""));
}

#[test]
fn test_empty_service_list() {
    let response = ServiceListResponse {
        count: 0,
        services: vec![],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"count\":0"));
    assert!(json.contains("\"services\":[]"));
}
