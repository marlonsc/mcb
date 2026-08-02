//! Issue entity CRUD: issue lifecycle operations.

use mcb_server::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use mcb_server::handlers::entities::IssueEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use rstest::rstest;

fn handler() -> TestResult<IssueEntityHandler> {
    let state = crate::utils::test_fixtures::shared_mcb_state()?;
    Ok(IssueEntityHandler::new(
        state.mcp_server.issue_entity_repository(),
    ))
}

fn issue_args(action: IssueEntityAction, project_id: Option<&str>) -> IssueEntityArgs {
    IssueEntityArgs {
        action,
        resource: IssueEntityResource::Issue,
        id: None,
        issue_id: None,
        label_id: None,
        org_id: None,
        project_id: project_id.map(ToOwned::to_owned),
        data: None,
    }
}

fn issue_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id, "org_id": "test-org", "project_id": project_id,
        "created_by": "user-1", "phase_id": null,
        "title": format!("Issue {id}"), "description": "test",
        "issue_type": "Task", "status": "Open", "priority": 2,
        "assignee": null, "labels": [], "estimated_minutes": null,
        "actual_minutes": null, "notes": "", "design": "",
        "parent_issue_id": null, "created_at": 1, "updated_at": 1,
        "closed_at": null, "closed_reason": ""
    })
}

async fn list_count(h: &IssueEntityHandler, project_id: &str) -> usize {
    let args = issue_args(IssueEntityAction::List, Some(project_id));
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
async fn list_issues_requires_project_id() -> TestResult {
    let h = handler()?;
    let err = h
        .handle(Parameters(issue_args(IssueEntityAction::List, None)))
        .await
        .expect_err("must reject missing project_id");
    assert!(err.message.contains("project_id required for list"));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn conflicting_project_id_rejected_without_side_effect() -> TestResult {
    let h = handler()?;
    let before = list_count(&h, "project-a").await;

    let mut args = issue_args(IssueEntityAction::Create, Some("project-a"));
    args.data = Some(issue_payload("issue-conflict", "project-b"));

    let err = h
        .handle(Parameters(args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));
    assert_eq!(list_count(&h, "project-a").await, before);
    Ok(())
}
