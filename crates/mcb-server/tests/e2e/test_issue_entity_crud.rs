use crate::utils::test_fixtures::*;
use mcb_server::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

fn base_args(action: IssueEntityAction, resource: IssueEntityResource) -> IssueEntityArgs {
    IssueEntityArgs {
        action,
        resource,
        id: None,
        org_id: None,
        project_id: None,
        issue_id: None,
        label_id: None,
        data: None,
    }
}

fn result_json(res: &rmcp::model::CallToolResult) -> serde_json::Value {
    let text = golden_content_to_string(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {text}; error: {e}"))
}

fn test_issue_payload(project_id: &str, title: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "org_id": "",
        "project_id": project_id,
        "created_by": "test-user",
        "title": title,
        "description": format!("Description for {title}"),
        "issue_type": "Task",
        "status": "Open",
        "priority": 2,
        "labels": [],
        "notes": "",
        "design": "",
        "closed_reason": "",
        "created_at": 0,
        "updated_at": 0
    })
}

fn test_comment_payload(issue_id: &str, content: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "issue_id": issue_id,
        "author_id": "test-user",
        "content": content,
        "created_at": 0
    })
}

fn test_label_payload(project_id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": uuid::Uuid::new_v4().clone(),
        "org_id": "",
        "project_id": project_id,
        "name": name,
        "color": "#ff0000",
        "created_at": 0
    })
}

async fn create_issue(
    server: &mcb_server::mcp_server::McpServer,
    project_id: &str,
    title: &str,
) -> serde_json::Value {
    let payload = test_issue_payload(project_id, title);

    let mut args = base_args(IssueEntityAction::Create, IssueEntityResource::Issue);
    args.project_id = Some(project_id.to_owned());
    args.data = Some(payload);

    let result = server.issue_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "issue create should succeed: {result:?}");
    result_json(&result.expect("issue create response"))
}

async fn create_comment(
    server: &mcb_server::mcp_server::McpServer,
    issue_id: &str,
    content: &str,
) -> serde_json::Value {
    let payload = test_comment_payload(issue_id, content);

    let mut args = base_args(IssueEntityAction::Create, IssueEntityResource::Comment);
    args.data = Some(payload);

    let result = server.issue_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "comment create should succeed: {result:?}");
    result_json(&result.expect("comment create response"))
}

async fn create_label(
    server: &mcb_server::mcp_server::McpServer,
    project_id: &str,
    name: &str,
) -> serde_json::Value {
    let payload = test_label_payload(project_id, name);

    let mut args = base_args(IssueEntityAction::Create, IssueEntityResource::Label);
    args.data = Some(payload);

    let result = server.issue_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "label create should succeed: {result:?}");
    result_json(&result.expect("label create response"))
}

// ============================================================================
// Issue CRUD (5 tests)
// ============================================================================

#[tokio::test]
async fn golden_issue_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-issue-create-get-proj";

    let created = create_issue(&server, project_id, "Golden Issue Create Get").await;
    let issue_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "created issue id must be present");

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Issue);
    get_args.id = Some(issue_id.clone());
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "issue get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("issue get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(issue_id.as_str())
    );
    assert_eq!(
        body.get("title").and_then(serde_json::Value::as_str),
        Some("Golden Issue Create Get")
    );
}

#[tokio::test]
async fn golden_issue_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-issue-list-proj";

    let _ = create_issue(&server, project_id, "Golden Issue List 1").await;
    let _ = create_issue(&server, project_id, "Golden Issue List 2").await;

    let mut list_args = base_args(IssueEntityAction::List, IssueEntityResource::Issue);
    list_args.project_id = Some(project_id.to_owned());
    let list_result = server
        .issue_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "issue list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("issue list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "issue list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_issue_update_status() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-issue-update-status-proj";

    let created = create_issue(&server, project_id, "Golden Issue Update Status").await;
    let issue_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "created issue id must be present");

    let mut updated = created.clone();
    updated["status"] = json!("InProgress");

    let mut update_args = base_args(IssueEntityAction::Update, IssueEntityResource::Issue);
    update_args.data = Some(updated);
    let update_result = server
        .issue_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "issue update should succeed: {update_result:?}"
    );

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Issue);
    get_args.id = Some(issue_id.clone());
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "issue get should succeed after update: {get_result:?}"
    );

    let body = result_json(&get_result.expect("issue get after update response"));
    assert_eq!(
        body.get("status").and_then(serde_json::Value::as_str),
        Some("InProgress")
    );
}

#[tokio::test]
async fn golden_issue_update_assignee() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-issue-update-assignee-proj";

    let created = create_issue(&server, project_id, "Golden Issue Update Assignee").await;
    let issue_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "created issue id must be present");

    let mut updated = created.clone();
    updated["assignee"] = json!("golden-assignee-user");

    let mut update_args = base_args(IssueEntityAction::Update, IssueEntityResource::Issue);
    update_args.data = Some(updated);
    let update_result = server
        .issue_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "issue update assignee should succeed: {update_result:?}"
    );

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Issue);
    get_args.id = Some(issue_id.clone());
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "issue get should succeed after assignee update: {get_result:?}"
    );

    let body = result_json(&get_result.expect("issue get after assignee update response"));
    assert_eq!(
        body.get("assignee").and_then(serde_json::Value::as_str),
        Some("golden-assignee-user")
    );
}

#[tokio::test]
async fn golden_issue_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-issue-delete-proj";

    let created = create_issue(&server, project_id, "Golden Issue Delete").await;
    let issue_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "created issue id must be present");

    let mut delete_args = base_args(IssueEntityAction::Delete, IssueEntityResource::Issue);
    delete_args.id = Some(issue_id.clone());
    let delete_result = server
        .issue_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "issue delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Issue);
    get_args.id = Some(issue_id);
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "issue get should fail after delete");
}

// ============================================================================
// IssueComment CRUD (3 tests)
// ============================================================================

#[tokio::test]
async fn golden_issue_comment_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-comment-create-get-proj";

    let issue = create_issue(&server, project_id, "Golden Comment Host Issue").await;
    let issue_id = issue
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "host issue id must be present");

    let created = create_comment(&server, &issue_id, "Golden comment content").await;
    let comment_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!comment_id.is_empty(), "created comment id must be present");

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Comment);
    get_args.id = Some(comment_id.clone());
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "comment get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("comment get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(comment_id.as_str())
    );
    assert_eq!(
        body.get("content").and_then(serde_json::Value::as_str),
        Some("Golden comment content")
    );
}

#[tokio::test]
async fn golden_issue_comment_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-comment-list-proj";

    let issue = create_issue(&server, project_id, "Golden Comment List Host").await;
    let issue_id = issue
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "host issue id must be present");

    let _ = create_comment(&server, &issue_id, "Comment list 1").await;
    let _ = create_comment(&server, &issue_id, "Comment list 2").await;

    let mut list_args = base_args(IssueEntityAction::List, IssueEntityResource::Comment);
    list_args.issue_id = Some(issue_id);
    let list_result = server
        .issue_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "comment list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("comment list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "comment list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_issue_comment_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-comment-delete-proj";

    let issue = create_issue(&server, project_id, "Golden Comment Delete Host").await;
    let issue_id = issue
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "host issue id must be present");

    let created = create_comment(&server, &issue_id, "Comment to delete").await;
    let comment_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!comment_id.is_empty(), "created comment id must be present");

    let mut delete_args = base_args(IssueEntityAction::Delete, IssueEntityResource::Comment);
    delete_args.id = Some(comment_id.clone());
    let delete_result = server
        .issue_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "comment delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Comment);
    get_args.id = Some(comment_id);
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "comment get should fail after delete");
}

// ============================================================================
// IssueLabel CRUD (3 tests)
// ============================================================================

#[tokio::test]
async fn golden_issue_label_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-label-create-get-proj";

    let created = create_label(&server, project_id, "golden-label-create-get").await;
    let label_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!label_id.is_empty(), "created label id must be present");

    let mut get_args = base_args(IssueEntityAction::Get, IssueEntityResource::Label);
    get_args.id = Some(label_id.clone());
    let get_result = server
        .issue_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "label get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("label get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(label_id.as_str())
    );
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some("golden-label-create-get")
    );
}

#[tokio::test]
async fn golden_issue_label_list() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-label-list-proj";

    let _ = create_label(&server, project_id, "golden-label-list-1").await;
    let _ = create_label(&server, project_id, "golden-label-list-2").await;

    let mut list_args = base_args(IssueEntityAction::List, IssueEntityResource::Label);
    list_args.project_id = Some(project_id.to_owned());
    let list_result = server
        .issue_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "label list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("label list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "label list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_issue_label_assign_to_issue() {
    let (server, _td) = create_test_mcp_server().await;
    let project_id = "golden-label-assign-proj";

    let issue = create_issue(&server, project_id, "Golden Label Assign Host").await;
    let issue_id = issue
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!issue_id.is_empty(), "host issue id must be present");

    let label = create_label(&server, project_id, "golden-assign-label").await;
    let label_id = label
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    assert!(!label_id.is_empty(), "created label id must be present");

    let assignment_payload = json!({
        "issue_id": issue_id,
        "label_id": label_id,
        "created_at": 0
    });

    let mut assign_args = base_args(
        IssueEntityAction::Create,
        IssueEntityResource::LabelAssignment,
    );
    assign_args.data = Some(assignment_payload);
    let assign_result = server
        .issue_entity_handler()
        .handle(Parameters(assign_args))
        .await;
    assert!(
        assign_result.is_ok(),
        "label assignment should succeed: {assign_result:?}"
    );

    let mut list_args = base_args(
        IssueEntityAction::List,
        IssueEntityResource::LabelAssignment,
    );
    list_args.issue_id = Some(issue_id);
    let list_result = server
        .issue_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "label assignment list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("label assignment list response"));
    let labels = body
        .as_array()
        .expect("labels response should be a JSON array")
        .clone();
    let has_label = labels.iter().any(|entry| {
        entry.get("id").and_then(serde_json::Value::as_str) == Some(label_id.as_str())
    });
    assert!(
        has_label,
        "label assignment list should include the assigned label"
    );
}

// ============================================================================
// Error paths (2 tests)
// ============================================================================

#[tokio::test]
async fn golden_issue_create_missing_fields() {
    let (server, _td) = create_test_mcp_server().await;

    let args = base_args(IssueEntityAction::Create, IssueEntityResource::Issue);
    let result = server.issue_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "issue create without data should fail");
}

#[tokio::test]
async fn golden_issue_get_nonexistent() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(IssueEntityAction::Get, IssueEntityResource::Issue);
    args.id = Some("00000000-0000-0000-0000-000000000000".to_owned());
    let result = server.issue_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "issue get with nonexistent id should fail");
}
