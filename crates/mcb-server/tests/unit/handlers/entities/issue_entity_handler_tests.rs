use mcb_server::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use mcb_server::handlers::entities::IssueEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::utils::text::extract_text;

fn create_handler() -> IssueEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    IssueEntityHandler::new(ctx.issue_entity_repository())
}

#[tokio::test]
async fn list_issues_requires_project_id() {
    let handler = create_handler();
    let args = IssueEntityArgs {
        action: IssueEntityAction::List,
        resource: IssueEntityResource::Issue,
        id: None,
        issue_id: None,
        label_id: None,
        org_id: None,
        project_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing project_id");
    assert!(err.message.contains("project_id required for list"));
}

fn issue_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "org_id": "ignored-org",
        "project_id": project_id,
        "created_by": "user-1",
        "phase_id": null,
        "title": format!("Issue {id}"),
        "description": "test issue",
        "issue_type": "Task",
        "status": "Open",
        "priority": 2,
        "assignee": null,
        "labels": [],
        "estimated_minutes": null,
        "actual_minutes": null,
        "notes": "",
        "design": "",
        "parent_issue_id": null,
        "created_at": 1,
        "updated_at": 1,
        "closed_at": null,
        "closed_reason": ""
    })
}

async fn list_issue_count(handler: &IssueEntityHandler, project_id: &str) -> usize {
    let list_args = IssueEntityArgs {
        action: IssueEntityAction::List,
        resource: IssueEntityResource::Issue,
        id: None,
        issue_id: None,
        label_id: None,
        org_id: None,
        project_id: Some(project_id.to_owned()),
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
async fn create_issue_with_conflicting_project_id_rejected_without_side_effect() {
    let handler = create_handler();
    let before_count = list_issue_count(&handler, "project-a").await;

    let create_args = IssueEntityArgs {
        action: IssueEntityAction::Create,
        resource: IssueEntityResource::Issue,
        id: None,
        issue_id: None,
        label_id: None,
        org_id: None,
        project_id: Some("project-a".to_owned()),
        data: Some(issue_payload("issue-conflict", "project-b")),
    };

    let err = handler
        .handle(Parameters(create_args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));

    let after_count = list_issue_count(&handler, "project-a").await;
    assert_eq!(after_count, before_count);
}
