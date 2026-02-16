use mcb_server::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use mcb_server::handlers::entities::PlanEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::utils::text::extract_text;

fn create_handler() -> PlanEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    PlanEntityHandler::new(ctx.plan_entity_repository())
}

#[tokio::test]
async fn list_plan_versions_requires_plan_id() {
    let handler = create_handler();
    let args = PlanEntityArgs {
        action: PlanEntityAction::List,
        resource: PlanEntityResource::Version,
        id: None,
        org_id: None,
        project_id: None,
        plan_id: None,
        plan_version_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing plan_id");
    assert!(err.message.contains("plan_id required"));
}

fn plan_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "created_at": 1,
        "updated_at": 1,
        "org_id": "ignored-org",
        "project_id": project_id,
        "title": format!("Plan {id}"),
        "description": "test plan",
        "status": "draft",
        "created_by": "user-1"
    })
}

async fn list_plan_count(handler: &PlanEntityHandler, project_id: &str) -> usize {
    let list_args = PlanEntityArgs {
        action: PlanEntityAction::List,
        resource: PlanEntityResource::Plan,
        id: None,
        org_id: None,
        project_id: Some(project_id.to_owned()),
        plan_id: None,
        plan_version_id: None,
        data: None,
    };
    let content = handler
        .handle(Parameters(list_args))
        .await
        .ok()
        .map(|r| r.content)
        .unwrap_or_default();
    let text = extract_text(&content);
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.as_array().map(std::vec::Vec::len))
        .unwrap_or(0)
}

#[tokio::test]
async fn create_plan_with_conflicting_project_id_rejected_without_side_effect() {
    let handler = create_handler();
    let before_count = list_plan_count(&handler, "project-a").await;

    let create_args = PlanEntityArgs {
        action: PlanEntityAction::Create,
        resource: PlanEntityResource::Plan,
        id: None,
        org_id: None,
        project_id: Some("project-a".to_owned()),
        plan_id: None,
        plan_version_id: None,
        data: Some(plan_payload("plan-conflict", "project-b")),
    };

    let err = handler
        .handle(Parameters(create_args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));

    let after_count = list_plan_count(&handler, "project-a").await;
    assert_eq!(after_count, before_count);
}
