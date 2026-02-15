//! Service Lifecycle Handlers Tests
//!
//! Tests for the service lifecycle management HTTP handlers.

use mcb_server::admin::lifecycle_handlers::{ServiceInfoResponse, ServiceListResponse};
use rstest::rstest;

#[rstest]
#[case(2, true)]
#[case(0, false)]
fn test_service_list_response_serialization(#[case] count: usize, #[case] with_services: bool) {
    let response = ServiceListResponse {
        count,
        services: if with_services {
            vec![
                ServiceInfoResponse {
                    name: "embedding".to_owned(),
                    state: "Running".to_owned(),
                },
                ServiceInfoResponse {
                    name: "vector_store".to_owned(),
                    state: "Stopped".to_owned(),
                },
            ]
        } else {
            vec![]
        },
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains(&format!("\"count\":{count}")));
    if with_services {
        assert!(json.contains("\"name\":\"embedding\""));
        assert!(json.contains("\"state\":\"Running\""));
    } else {
        assert!(json.contains("\"services\":[]"));
    }
}

#[test]
fn test_service_info_response_serialization() {
    let info = ServiceInfoResponse {
        name: "cache".to_owned(),
        state: "Starting".to_owned(),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"name\":\"cache\""));
    assert!(json.contains("\"state\":\"Starting\""));
}
