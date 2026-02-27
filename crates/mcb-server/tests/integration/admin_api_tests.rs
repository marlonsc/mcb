#![allow(clippy::expect_used, clippy::doc_markdown)]
//! Admin API controller integration tests (T6: Health, T7: Jobs, T8: Collections).
//!
//! Tests use `create_real_domain_services()` to build a full McbState with real
//! SQLite database, FastEmbed embeddings, and in-memory vector store. This matches
//! the existing handler integration test pattern.

use axum::extract::Extension;
use http_body_util::BodyExt;
use serde_json::Value;

use crate::utils::domain_services::create_real_domain_services;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract JSON body from an Axum response.
async fn json_body(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("parse JSON body")
}

// ---------------------------------------------------------------------------
// T6: Health API
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint_returns_json_with_status() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return; // skip if providers not available
    };

    let response = mcb_server::controllers::health_api::health(Extension(state))
        .await
        .expect("health handler should succeed");

    let body = json_body(response).await;

    // Must have a status field
    let status = body["status"].as_str().expect("status is string");
    assert!(
        status == "healthy" || status == "degraded",
        "status must be healthy or degraded, got: {status}"
    );
}

#[tokio::test]
async fn test_health_endpoint_includes_provider_metadata() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::health_api::health(Extension(state))
        .await
        .expect("health handler should succeed");

    let body = json_body(response).await;

    // Embedding section must have provider name and dimensions
    assert!(
        body["embedding"]["provider"].is_string(),
        "embedding.provider must be a string"
    );
    assert!(
        body["embedding"]["dimensions"].is_number(),
        "embedding.dimensions must be a number"
    );
    assert!(
        body["embedding"]["healthy"].is_boolean(),
        "embedding.healthy must be a boolean"
    );

    // Vector store section must have healthy field
    assert!(
        body["vector_store"]["healthy"].is_boolean(),
        "vector_store.healthy must be a boolean"
    );
}

#[tokio::test]
async fn test_health_endpoint_provider_name_is_nonempty() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::health_api::health(Extension(state))
        .await
        .expect("health handler should succeed");

    let body = json_body(response).await;
    let provider = body["embedding"]["provider"]
        .as_str()
        .expect("provider is string");
    assert!(!provider.is_empty(), "provider name must not be empty");
}

// ---------------------------------------------------------------------------
// T7: Jobs API
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_jobs_endpoint_returns_empty_operations() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::jobs_api::jobs(Extension(state))
        .await
        .expect("jobs handler should succeed");

    let body = json_body(response).await;

    // Fresh server should have zero operations
    assert_eq!(body["total"].as_u64(), Some(0), "total should be 0");
    assert_eq!(body["running"].as_u64(), Some(0), "running should be 0");
    assert_eq!(body["queued"].as_u64(), Some(0), "queued should be 0");
}

#[tokio::test]
async fn test_jobs_endpoint_json_structure() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::jobs_api::jobs(Extension(state))
        .await
        .expect("jobs handler should succeed");

    let body = json_body(response).await;

    // Verify all expected top-level fields exist
    assert!(body["total"].is_number(), "total must be a number");
    assert!(body["running"].is_number(), "running must be a number");
    assert!(body["queued"].is_number(), "queued must be a number");
    assert!(body["jobs"].is_array(), "jobs must be an array");
    assert!(
        body["indexing_operations"].is_array(),
        "indexing_operations must be an array"
    );
    assert!(
        body["validation_operations"].is_array(),
        "validation_operations must be an array"
    );
}

#[tokio::test]
async fn test_jobs_total_matches_operations_count() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::jobs_api::jobs(Extension(state))
        .await
        .expect("jobs handler should succeed");

    let body = json_body(response).await;
    let total = body["total"].as_u64().unwrap_or(0);
    let indexing_count = body["indexing_operations"]
        .as_array()
        .map_or(0, |a| a.len() as u64);
    let validation_count = body["validation_operations"]
        .as_array()
        .map_or(0, |a| a.len() as u64);

    assert_eq!(
        total,
        indexing_count + validation_count,
        "total must equal indexing + validation count"
    );
}

// ---------------------------------------------------------------------------
// T8: Collections API
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_collections_endpoint_returns_list() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    let response = mcb_server::controllers::collections_api::collections(Extension(state))
        .await
        .expect("collections handler should succeed");

    let body = json_body(response).await;

    // Response must be a JSON array (may be empty on fresh server)
    assert!(body.is_array(), "collections response must be an array");
}

#[tokio::test]
async fn test_collections_endpoint_graceful_on_fresh_server() {
    let Some((state, _tmp)) = create_real_domain_services().await else {
        return;
    };

    // Even if vector store has no collections, response should succeed (not 500)
    let result = mcb_server::controllers::collections_api::collections(Extension(state)).await;
    assert!(
        result.is_ok(),
        "collections should succeed even with no collections"
    );
}
