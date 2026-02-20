use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use rocket::http::Status;

use crate::utils::timeouts::TEST_TIMEOUT;

use crate::utils::admin_harness::AdminTestHarness;

#[rocket::async_test]
async fn test_full_admin_stack_integration() {
    let (client, metrics, indexing) = AdminTestHarness::new().build_client().await;

    let response = client.get("/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["active_indexing_operations"], 0);

    metrics.record_query(100, true, true);
    metrics.record_query(150, true, false);
    metrics.record_query(200, false, false);
    metrics.update_active_connections(5);

    let response = client.get("/metrics").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 3);
    assert_eq!(json["successful_queries"], 2);
    assert_eq!(json["failed_queries"], 1);
    assert_eq!(json["active_connections"], 5);
    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.333).abs() < 0.01,
        "Expected ~33% cache hit rate, got {cache_hit_rate}"
    );

    let op1 = indexing.start_operation(&CollectionId::from_name("project-alpha"), 100);
    let _op2 = indexing.start_operation(&CollectionId::from_name("project-beta"), 200);
    indexing.update_progress(&op1, Some("src/main.rs".to_owned()), 25);

    // 6. Verify jobs endpoint shows operations
    let response = client.get("/jobs").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["running"], 2);
    assert_eq!(json["queued"], 0);
    assert_eq!(json["total"], 2);

    let ops = json["jobs"].as_array().unwrap();
    assert_eq!(ops.len(), 2);

    let expected_alpha_label = CollectionId::from_name("project-alpha").to_string();

    // Find the project-alpha operation
    let alpha_op = ops
        .iter()
        .find(|op| op["label"].as_str() == Some(expected_alpha_label.as_str()))
        .expect("Should find project-alpha operation");
    assert_eq!(alpha_op["processed_items"], 25);
    assert_eq!(alpha_op["total_items"], 100);
    assert_eq!(alpha_op["progress_percent"], 25);
    assert_eq!(alpha_op["current_item"], "src/main.rs");

    // 7. Health should now show active indexing operations
    let response = client.get("/health").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["active_indexing_operations"], 2);

    // 8. Complete one operation
    indexing.complete_operation(&op1);

    let response = client.get("/jobs").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["running"], 1);

    // 9. Verify liveness probe (always OK)
    let response = client.get("/live").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["alive"], true);

    // 10. Wait for readiness (needs uptime > 1s)
    tokio::time::timeout(TEST_TIMEOUT, async {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    })
    .await
    .expect("readiness wait timed out");

    let response = client.get("/ready").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["ready"], true);
}

#[rocket::async_test]
async fn test_metrics_accumulation_integration() {
    let (client, metrics, _) = AdminTestHarness::new().build_client().await;

    for _ in 0..10 {
        metrics.record_query(50, true, true);
    }

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["total_queries"], 10);
    assert_eq!(json["cache_hit_rate"], 1.0);

    for _ in 0..5 {
        metrics.record_query(100, false, false);
    }

    let response = client.get("/metrics").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["total_queries"], 15);
    assert_eq!(json["successful_queries"], 10);
    assert_eq!(json["failed_queries"], 5);
    let cache_hit_rate = json["cache_hit_rate"].as_f64().unwrap();
    assert!(
        (cache_hit_rate - 0.666).abs() < 0.01,
        "Expected ~66.6% cache hit rate"
    );
}

#[rocket::async_test]
async fn test_indexing_lifecycle_integration() {
    let (client, _, indexing) = AdminTestHarness::new().build_client().await;

    let op1 = indexing.start_operation(&CollectionId::from_name("repo-1"), 50);
    let op2 = indexing.start_operation(&CollectionId::from_name("repo-2"), 100);
    let op3 = indexing.start_operation(&CollectionId::from_name("repo-3"), 150);

    let response = client.get("/jobs").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["running"], 3);

    indexing.update_progress(&op1, Some("file1.rs".to_owned()), 50);
    indexing.update_progress(&op2, Some("file2.rs".to_owned()), 50);
    indexing.update_progress(&op3, Some("file3.rs".to_owned()), 75);

    indexing.complete_operation(&op1);

    let response = client.get("/jobs").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["running"], 2);

    indexing.complete_operation(&op2);
    indexing.complete_operation(&op3);

    let response = client.get("/jobs").dispatch().await;

    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["running"], 0);
    assert_eq!(json["total"], 0);
}
