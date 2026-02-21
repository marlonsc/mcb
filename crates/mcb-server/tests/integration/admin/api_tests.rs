use mcb_domain::ports::IndexingOperationsInterface;
use mcb_domain::value_objects::CollectionId;
use rocket::http::Status;

use crate::utils::timeouts::TEST_TIMEOUT;

use crate::utils::admin_harness::AdminTestHarness;

#[rocket::async_test]
async fn test_health_endpoint() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert!(json["uptime_seconds"].is_number());
    assert_eq!(json["active_indexing_operations"], 0);
}

#[rocket::async_test]
async fn test_metrics_endpoint() {
    let (client, _, _) = AdminTestHarness::new()
        .with_recorded_metrics(&[(100, true, true), (200, false, false)], 3)
        .build_client()
        .await;

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 2);
    assert_eq!(json["successful_queries"], 1);
    assert_eq!(json["failed_queries"], 1);
    assert_eq!(json["active_connections"], 3);
    assert!(json["average_response_time_ms"].as_f64().unwrap() > 0.0);
}

#[rocket::async_test]
async fn test_jobs_endpoint_no_operations() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/jobs").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["running"], 0);
    assert_eq!(json["queued"], 0);
    assert_eq!(json["total"], 0);
    assert!(json["jobs"].as_array().unwrap().is_empty());
}

#[rocket::async_test]
async fn test_jobs_endpoint_with_operations() {
    let harness = AdminTestHarness::new();
    let op_id = harness
        .indexing()
        .start_operation(&CollectionId::from_name("test-collection"), 50);
    harness
        .indexing()
        .update_progress(&op_id, Some("src/main.rs".to_owned()), 10);
    let (client, _, _) = harness.build_client().await;

    let response = client.get("/jobs").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["running"], 1);
    assert_eq!(json["queued"], 0);
    assert_eq!(json["total"], 1);

    let ops = json["jobs"].as_array().unwrap();
    assert_eq!(ops.len(), 1);

    let op = &ops[0];
    assert_eq!(
        op["label"],
        CollectionId::from_name("test-collection").to_string()
    );
    assert_eq!(op["current_item"], "src/main.rs");
    assert_eq!(op["processed_items"], 10);
    assert_eq!(op["total_items"], 50);
    assert_eq!(op["progress_percent"], 20);
}

#[rocket::async_test]
async fn test_readiness_probe_not_ready() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ready").dispatch().await;

    let status = response.status();
    assert!(
        status == Status::Ok || status == Status::ServiceUnavailable,
        "Expected Ok or ServiceUnavailable, got {status:?}"
    );

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    // The ready field should be boolean
    assert!(json["ready"].is_boolean());
}

#[rocket::async_test]
async fn test_readiness_probe_ready() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;
    tokio::time::timeout(TEST_TIMEOUT, async {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    })
    .await
    .expect("readiness setup timed out");

    let response = client.get("/ready").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["ready"], true);
}

#[rocket::async_test]
async fn test_liveness_probe() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/live").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["alive"], true);
}

#[rocket::async_test]
async fn test_health_with_active_operations() {
    let (client, _, _) = AdminTestHarness::new()
        .with_indexing_operations(&[("coll-1", 100), ("coll-2", 200)])
        .build_client()
        .await;

    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["active_indexing_operations"], 2);
}

#[rocket::async_test]
async fn test_metrics_with_cache_hits() {
    let (client, _, _) = AdminTestHarness::new()
        .with_recorded_metrics(
            &[
                (10, true, true),
                (20, true, true),
                (30, true, true),
                (40, true, false),
                (50, true, false),
            ],
            0,
        )
        .build_client()
        .await;

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.6).abs() < 0.01,
        "Expected ~60% cache hit rate"
    );
}
