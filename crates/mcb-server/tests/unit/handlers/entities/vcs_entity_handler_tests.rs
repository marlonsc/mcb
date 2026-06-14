//! VCS entity CRUD: repository lifecycle operations.
//!
//! Tests verify validation rules for repository create/update/delete/list
//! including project_id requirement and conflict detection.

use mcb_server::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use mcb_server::handlers::entities::VcsEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use rstest::rstest;

fn handler() -> TestResult<VcsEntityHandler> {
    let state = crate::utils::test_fixtures::shared_mcb_state()?;
    Ok(VcsEntityHandler::new(
        state.mcp_server.vcs_entity_repository(),
    ))
}

fn vcs_args(action: VcsEntityAction, project_id: Option<&str>) -> VcsEntityArgs {
    VcsEntityArgs {
        action,
        resource: VcsEntityResource::Repository,
        id: None,
        org_id: None,
        project_id: project_id.map(ToOwned::to_owned),
        repository_id: None,
        worktree_id: None,
        data: None,
    }
}

fn repo_payload(id: &str, project_id: &str) -> serde_json::Value {
    json!({
        "id": id, "org_id": "test-org", "project_id": project_id,
        "name": format!("repo-{id}"), "url": "https://example.com/repo.git",
        "local_path": format!("/tmp/{id}"), "vcs_type": "git",
        "created_at": 1, "updated_at": 1
    })
}

async fn list_count(h: &VcsEntityHandler, project_id: &str) -> usize {
    let args = vcs_args(VcsEntityAction::List, Some(project_id));
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

// ─── Required fields ─────────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn list_repositories_requires_project_id() -> TestResult {
    let h = handler()?;
    let err = h
        .handle(Parameters(vcs_args(VcsEntityAction::List, None)))
        .await
        .expect_err("must reject missing project_id");
    assert!(err.message.contains("project_id required for list"));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_repository_requires_project_id() -> TestResult {
    let h = handler()?;
    let mut args = vcs_args(VcsEntityAction::Delete, None);
    args.id = Some("repo-any".to_owned());
    let err = h
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing project_id");
    assert!(
        err.message
            .contains("project_id required for repository delete")
    );
    Ok(())
}

// ─── Conflict detection ──────────────────────────────────────────────

#[rstest]
#[case(VcsEntityAction::Create, "repo-create-conflict")]
#[case(VcsEntityAction::Update, "repo-update-conflict")]
#[tokio::test]
async fn conflicting_project_id_rejected_without_side_effect(
    #[case] action: VcsEntityAction,
    #[case] id: &str,
) -> TestResult {
    let h = handler()?;
    let before = list_count(&h, "project-a").await;

    let mut args = vcs_args(action, Some("project-a"));
    args.data = Some(repo_payload(id, "project-b"));

    let err = h
        .handle(Parameters(args))
        .await
        .expect_err("conflicting project_id must fail");
    assert!(err.message.contains("conflicting project_id"));
    assert_eq!(list_count(&h, "project-a").await, before);
    Ok(())
}
