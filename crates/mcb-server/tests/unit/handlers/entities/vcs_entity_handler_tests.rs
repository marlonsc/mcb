use mcb_server::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use mcb_server::handlers::entities::VcsEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::text::extract_text;

fn create_handler() -> VcsEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    VcsEntityHandler::new(ctx.vcs_entity_repository())
}

#[tokio::test]
async fn list_repositories_requires_project_id() {
    let handler = create_handler();
    let args = VcsEntityArgs {
        action: VcsEntityAction::List,
        resource: VcsEntityResource::Repository,
        id: None,
        org_id: None,
        project_id: None,
        repository_id: None,
        worktree_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing project_id");
    assert!(err.message.contains("project_id required for list"));
}

fn repo_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "org_id": "ignored-org",
        "project_id": project_id,
        "name": format!("repo-{id}"),
        "url": "https://example.com/repo.git",
        "local_path": format!("/tmp/{id}"),
        "vcs_type": "git",
        "created_at": 1,
        "updated_at": 1
    })
}

async fn list_repo_count(handler: &VcsEntityHandler, project_id: &str) -> usize {
    let list_args = VcsEntityArgs {
        action: VcsEntityAction::List,
        resource: VcsEntityResource::Repository,
        id: None,
        org_id: None,
        project_id: Some(project_id.to_owned()),
        repository_id: None,
        worktree_id: None,
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
async fn create_repository_conflicting_project_id_rejected_without_side_effect() {
    let handler = create_handler();
    let before_count = list_repo_count(&handler, "project-a").await;

    let create_args = VcsEntityArgs {
        action: VcsEntityAction::Create,
        resource: VcsEntityResource::Repository,
        id: None,
        org_id: None,
        project_id: Some("project-a".to_owned()),
        repository_id: None,
        worktree_id: None,
        data: Some(repo_payload("repo-conflict", "project-b")),
    };

    let err = handler
        .handle(Parameters(create_args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));

    let after_count = list_repo_count(&handler, "project-a").await;

    assert_eq!(after_count, before_count);
}

#[tokio::test]
async fn update_repository_conflicting_project_id_rejected_without_side_effect() {
    let handler = create_handler();

    let before_count = list_repo_count(&handler, "project-a").await;

    let update_args = VcsEntityArgs {
        action: VcsEntityAction::Update,
        resource: VcsEntityResource::Repository,
        id: None,
        org_id: None,
        project_id: Some("project-a".to_owned()),
        repository_id: None,
        worktree_id: None,
        data: Some(repo_payload("repo-update-conflict", "project-b")),
    };
    let err = handler
        .handle(Parameters(update_args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));

    let after_count = list_repo_count(&handler, "project-a").await;
    assert_eq!(after_count, before_count);
}

#[tokio::test]
async fn delete_repository_requires_project_id() {
    let handler = create_handler();

    let delete_args = VcsEntityArgs {
        action: VcsEntityAction::Delete,
        resource: VcsEntityResource::Repository,
        id: Some("repo-any".to_owned()),
        org_id: None,
        project_id: None,
        repository_id: None,
        worktree_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(delete_args))
        .await
        .expect_err("missing project_id must fail");
    assert!(
        err.message
            .contains("project_id required for repository delete")
    );
}
