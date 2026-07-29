//! Plan entity CRUD: plan and version lifecycle operations.

use mcb_server::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use mcb_server::handlers::entities::PlanEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use rstest::rstest;

fn handler() -> TestResult<PlanEntityHandler> {
    let state = crate::utils::test_fixtures::shared_mcb_state()?;
    Ok(PlanEntityHandler::new(
        state.mcp_server.plan_entity_repository(),
    ))
}

fn plan_args(action: PlanEntityAction, resource: PlanEntityResource) -> PlanEntityArgs {
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

fn plan_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id, "org_id": "test-org", "project_id": project_id,
        "title": format!("Plan {id}"), "description": "test",
        "status": "draft", "created_by": "user-1",
        "created_at": 1, "updated_at": 1
    })
}

async fn list_count(h: &PlanEntityHandler, project_id: &str) -> usize {
    let mut args = plan_args(PlanEntityAction::List, PlanEntityResource::Plan);
    args.project_id = Some(project_id.to_owned());
    let content = h
        .handle(Parameters(args))
        .await
        .ok()
        .map(|r| r.content)
        .unwrap_or_default();
    let text = extract_text_from(&content);
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.as_array().map(Vec::len))
        .unwrap_or(0)
}

#[rstest]
#[tokio::test]
async fn list_plan_versions_requires_plan_id() -> TestResult {
    let h = handler()?;
    let err = h
        .handle(Parameters(plan_args(
            PlanEntityAction::List,
            PlanEntityResource::Version,
        )))
        .await
        .expect_err("must reject missing plan_id");
    assert!(err.message.contains("plan_id required"));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn conflicting_project_id_rejected_without_side_effect() -> TestResult {
    let h = handler()?;
    let before = list_count(&h, "project-a").await;

    let mut args = plan_args(PlanEntityAction::Create, PlanEntityResource::Plan);
    args.project_id = Some("project-a".to_owned());
    args.data = Some(plan_payload("plan-conflict", "project-b"));

    let err = h
        .handle(Parameters(args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));
    assert_eq!(list_count(&h, "project-a").await, before);
    Ok(())
}
