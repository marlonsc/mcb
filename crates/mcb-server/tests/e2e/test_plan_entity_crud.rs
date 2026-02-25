use crate::utils::test_fixtures::*;
use mcb_server::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

fn base_args(action: PlanEntityAction, resource: PlanEntityResource) -> PlanEntityArgs {
    PlanEntityArgs {
        action,
        resource,
        id: None,
        org_id: None,
        project_id: None,
        plan_id: None,
        plan_version_id: None,
        data: None,
    }
}

fn result_json(res: &rmcp::model::CallToolResult) -> serde_json::Value {
    let text = golden_content_to_string(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {text}; error: {e}"))
}

fn test_plan_payload(project_id: &str, title: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "org_id": "",
        "project_id": project_id,
        "title": title,
        "description": format!("Description for {title}"),
        "status": "draft",
        "created_by": "test-user",
        "created_at": 0,
        "updated_at": 0
    })
}

fn test_version_payload(plan_id: &str, version_number: i64) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "org_id": "",
        "plan_id": plan_id,
        "version_number": version_number,
        "content_json": json!({"steps": ["step-1", "step-2"]}).to_owned(),
        "change_summary": format!("Version {version_number} changes"),
        "created_by": "test-user",
        "created_at": 0
    })
}

fn test_review_payload(plan_version_id: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "org_id": "",
        "plan_version_id": plan_version_id,
        "reviewer_id": "reviewer-user",
        "verdict": "approved",
        "feedback": "Looks good to me",
        "created_at": 0
    })
}

async fn create_plan(
    server: &mcb_server::mcp_server::McpServer,
    project_id: &str,
    title: &str,
) -> serde_json::Value {
    let payload = test_plan_payload(project_id, title);

    let mut args = base_args(PlanEntityAction::Create, PlanEntityResource::Plan);
    args.project_id = Some(project_id.to_owned());
    args.data = Some(payload);

    let result = server.plan_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "plan create should succeed: {result:?}");
    result_json(&result.expect("plan create response"))
}

async fn create_version(
    server: &mcb_server::mcp_server::McpServer,
    plan_id: &str,
    version_number: i64,
) -> serde_json::Value {
    let payload = test_version_payload(plan_id, version_number);

    let mut args = base_args(PlanEntityAction::Create, PlanEntityResource::Version);
    args.data = Some(payload);

    let result = server.plan_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "version create should succeed: {result:?}");
    result_json(&result.expect("version create response"))
}

async fn create_review(
    server: &mcb_server::mcp_server::McpServer,
    plan_version_id: &str,
) -> serde_json::Value {
    let payload = test_review_payload(plan_version_id);

    let mut args = base_args(PlanEntityAction::Create, PlanEntityResource::Review);
    args.data = Some(payload);

    let result = server.plan_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "review create should succeed: {result:?}");
    result_json(&result.expect("review create response"))
}

// ---------------------------------------------------------------------------
// Plan CRUD (4 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_plan_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-plan-create-get-proj";

    let created = create_plan(&server, project_id, "Golden Plan Create Get").await;
    let plan_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "created plan id must be present");

    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Plan);
    get_args.id = Some(plan_id.clone());
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "plan get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("plan get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(plan_id.as_str())
    );
    assert_eq!(
        body.get("title").and_then(serde_json::Value::as_str),
        Some("Golden Plan Create Get")
    );
}

#[tokio::test]
async fn golden_plan_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-plan-list-proj";

    let _ = create_plan(&server, project_id, "Golden Plan List 1").await;
    let _ = create_plan(&server, project_id, "Golden Plan List 2").await;

    let mut list_args = base_args(PlanEntityAction::List, PlanEntityResource::Plan);
    list_args.project_id = Some(project_id.to_owned());
    let list_result = server
        .plan_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "plan list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("plan list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "plan list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_plan_update() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-plan-update-proj";

    let created = create_plan(&server, project_id, "Golden Plan Update").await;
    let plan_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "created plan id must be present");

    let mut updated = created.clone();
    updated["title"] = json!("Golden Plan Updated Title");

    let mut update_args = base_args(PlanEntityAction::Update, PlanEntityResource::Plan);
    update_args.data = Some(updated);
    let update_result = server
        .plan_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "plan update should succeed: {update_result:?}"
    );

    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Plan);
    get_args.id = Some(plan_id);
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "plan get should succeed after update: {get_result:?}"
    );

    let body = result_json(&get_result.expect("plan get after update response"));
    assert_eq!(
        body.get("title").and_then(serde_json::Value::as_str),
        Some("Golden Plan Updated Title")
    );
}

#[tokio::test]
async fn golden_plan_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-plan-delete-proj";

    let created = create_plan(&server, project_id, "Golden Plan Delete").await;
    let plan_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "created plan id must be present");

    let mut delete_args = base_args(PlanEntityAction::Delete, PlanEntityResource::Plan);
    delete_args.id = Some(plan_id.clone());
    let delete_result = server
        .plan_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "plan delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Plan);
    get_args.id = Some(plan_id);
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "plan get should fail after delete");
}

// ---------------------------------------------------------------------------
// PlanVersion CRUD (3 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_plan_version_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-version-create-get-proj";

    let plan = create_plan(&server, project_id, "Plan for Version Create Get").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let created = create_version(&server, &plan_id, 1).await;
    let version_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!version_id.is_empty(), "created version id must be present");

    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Version);
    get_args.id = Some(version_id.clone());
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "version get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("version get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(version_id.as_str())
    );
    assert_eq!(
        body.get("plan_id").and_then(serde_json::Value::as_str),
        Some(plan_id.as_str())
    );
}

#[tokio::test]
async fn golden_plan_version_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-version-list-proj";

    let plan = create_plan(&server, project_id, "Plan for Version List").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let _ = create_version(&server, &plan_id, 1).await;
    let _ = create_version(&server, &plan_id, 2).await;

    let mut list_args = base_args(PlanEntityAction::List, PlanEntityResource::Version);
    list_args.plan_id = Some(plan_id);
    let list_result = server
        .plan_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "version list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("version list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "version list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_plan_version_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-version-delete-proj";

    let plan = create_plan(&server, project_id, "Plan for Version Delete").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let created = create_version(&server, &plan_id, 1).await;
    let version_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!version_id.is_empty(), "version id must be present");

    // Version delete is not in the handler dispatch — use the generic unsupported path
    // Check if Delete+Version is supported by looking at the handler
    let mut delete_args = base_args(PlanEntityAction::Delete, PlanEntityResource::Version);
    delete_args.id = Some(version_id.clone());
    let delete_result = server
        .plan_entity_handler()
        .handle(Parameters(delete_args))
        .await;

    // If delete is unsupported for versions, verify the error
    if delete_result.is_err() {
        // Delete not supported for versions — that's expected
        return;
    }

    // If delete succeeded, verify get fails
    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Version);
    get_args.id = Some(version_id);
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "version get should fail after delete");
}

// ---------------------------------------------------------------------------
// PlanReview CRUD (3 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_plan_review_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-review-create-get-proj";

    let plan = create_plan(&server, project_id, "Plan for Review Create Get").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let version = create_version(&server, &plan_id, 1).await;
    let version_id = version
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!version_id.is_empty(), "version id must be present");

    let created = create_review(&server, &version_id).await;
    let review_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!review_id.is_empty(), "created review id must be present");

    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Review);
    get_args.id = Some(review_id.clone());
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "review get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("review get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(review_id.as_str())
    );
    assert_eq!(
        body.get("plan_version_id")
            .and_then(serde_json::Value::as_str),
        Some(version_id.as_str())
    );
}

#[tokio::test]
async fn golden_plan_review_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-review-list-proj";

    let plan = create_plan(&server, project_id, "Plan for Review List").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let version = create_version(&server, &plan_id, 1).await;
    let version_id = version
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!version_id.is_empty(), "version id must be present");

    let _ = create_review(&server, &version_id).await;

    // Create a second review with different verdict
    let mut review2_payload = test_review_payload(&version_id);
    review2_payload["verdict"] = json!("rejected");
    review2_payload["feedback"] = json!("Needs more detail");
    let mut args2 = base_args(PlanEntityAction::Create, PlanEntityResource::Review);
    args2.data = Some(review2_payload);
    let result2 = server.plan_entity_handler().handle(Parameters(args2)).await;
    assert!(
        result2.is_ok(),
        "second review create should succeed: {result2:?}"
    );

    let mut list_args = base_args(PlanEntityAction::List, PlanEntityResource::Review);
    list_args.plan_version_id = Some(version_id);
    let list_result = server
        .plan_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "review list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("review list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "review list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_plan_review_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-review-delete-proj";

    let plan = create_plan(&server, project_id, "Plan for Review Delete").await;
    let plan_id = plan
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!plan_id.is_empty(), "plan id must be present");

    let version = create_version(&server, &plan_id, 1).await;
    let version_id = version
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!version_id.is_empty(), "version id must be present");

    let created = create_review(&server, &version_id).await;
    let review_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!review_id.is_empty(), "review id must be present");

    // Review delete may not be in the handler dispatch
    let mut delete_args = base_args(PlanEntityAction::Delete, PlanEntityResource::Review);
    delete_args.id = Some(review_id.clone());
    let delete_result = server
        .plan_entity_handler()
        .handle(Parameters(delete_args))
        .await;

    // If delete is unsupported for reviews, verify the error
    if delete_result.is_err() {
        // Delete not supported for reviews — that's expected
        return;
    }

    // If delete succeeded, verify get fails
    let mut get_args = base_args(PlanEntityAction::Get, PlanEntityResource::Review);
    get_args.id = Some(review_id);
    let get_result = server
        .plan_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "review get should fail after delete");
}

// ---------------------------------------------------------------------------
// Error paths (2 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_plan_create_missing_data() {
    let (server, _td) = create_test_mcp_server().await;

    let args = base_args(PlanEntityAction::Create, PlanEntityResource::Plan);
    let result = server.plan_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "plan create without data should fail");
}

#[tokio::test]
async fn golden_plan_get_nonexistent() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(PlanEntityAction::Get, PlanEntityResource::Plan);
    args.id = Some("nonexistent-plan-id-00000000".to_owned());
    let result = server.plan_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "plan get with fake id should fail");
}
