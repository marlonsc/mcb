use crate::utils::test_fixtures::*;
use mcb_server::args::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

fn base_args(action: VcsEntityAction, resource: VcsEntityResource) -> VcsEntityArgs {
    VcsEntityArgs {
        action,
        resource,
        id: None,
        org_id: None,
        project_id: None,
        repository_id: None,
        worktree_id: None,
        data: None,
    }
}

fn result_json(res: &rmcp::model::CallToolResult) -> serde_json::Value {
    let text = golden_content_to_string(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {text}; error: {e}"))
}

/// Helper: create a repository and return the JSON response body.
async fn create_repo(
    server: &mcb_server::mcp_server::McpServer,
    org_id: &str,
    project_id: &str,
    repo_id: &str,
) -> serde_json::Value {
    let payload = json!({
        "id": repo_id,
        "org_id": org_id,
        "project_id": project_id,
        "created_at": 0,
        "updated_at": 0,
        "name": format!("repo-{repo_id}"),
        "url": format!("https://github.com/test/{repo_id}.git"),
        "local_path": format!("/tmp/{repo_id}"),
        "vcs_type": "git"
    });

    let mut args = base_args(VcsEntityAction::Create, VcsEntityResource::Repository);
    args.org_id = Some(org_id.to_owned());
    args.project_id = Some(project_id.to_owned());
    args.data = Some(payload);

    let result = server.vcs_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "repo create should succeed: {result:?}");
    let result_ok = match result {
        Ok(r) => r,
        Err(e) => panic!("repo create response: {e}"),
    };
    result_json(&result_ok)
}

/// Helper: create a branch and return the JSON response body.
async fn create_branch(
    server: &mcb_server::mcp_server::McpServer,
    repo_id: &str,
    branch_id: &str,
    branch_name: &str,
) -> serde_json::Value {
    let payload = json!({
        "id": branch_id,
        "org_id": "default",
        "created_at": 0,
        "repository_id": repo_id,
        "name": branch_name,
        "is_default": false,
        "head_commit": "abc123def456",
        "upstream": null
    });

    let mut args = base_args(VcsEntityAction::Create, VcsEntityResource::Branch);
    args.data = Some(payload);

    let result = server.vcs_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "branch create should succeed: {result:?}");
    let result_ok = match result {
        Ok(r) => r,
        Err(e) => panic!("branch create response: {e}"),
    };
    result_json(&result_ok)
}

/// Helper: create a worktree and return the JSON response body.
async fn create_worktree(
    server: &mcb_server::mcp_server::McpServer,
    repo_id: &str,
    branch_id: &str,
    worktree_id: &str,
) -> serde_json::Value {
    let payload = json!({
        "id": worktree_id,
        "created_at": 0,
        "updated_at": 0,
        "repository_id": repo_id,
        "branch_id": branch_id,
        "path": format!("/tmp/worktrees/{worktree_id}"),
        "status": "active",
        "assigned_agent_id": null
    });

    let mut args = base_args(VcsEntityAction::Create, VcsEntityResource::Worktree);
    args.data = Some(payload);

    let result = server.vcs_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "worktree create should succeed: {result:?}");
    let result_ok = match result {
        Ok(r) => r,
        Err(e) => panic!("worktree create response: {e}"),
    };
    result_json(&result_ok)
}

// ---------------------------------------------------------------------------
// Repository CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_vcs_repo_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-repo-cg";
    let project_id = "golden-vcs-proj-cg";
    let repo_id = "golden-vcs-repo-cg-1";

    let created = create_repo(&server, org_id, project_id, repo_id).await;
    assert_eq!(
        created.get("id").and_then(serde_json::Value::as_str),
        Some(repo_id)
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Repository);
    get_args.id = Some(repo_id.to_owned());
    get_args.org_id = Some(org_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "repo get should succeed: {get_result:?}"
    );

    let body = result_json(&match get_result {
        Ok(r) => r,
        Err(e) => panic!("repo get response: {e}"),
    });
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(repo_id)
    );
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some(format!("repo-{repo_id}").as_str())
    );
}

#[tokio::test]
async fn golden_vcs_repo_list() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-repo-list";
    let project_id = "golden-vcs-proj-list";

    let _ = create_repo(&server, org_id, project_id, "golden-vcs-repo-list-1").await;
    let _ = create_repo(&server, org_id, project_id, "golden-vcs-repo-list-2").await;

    let mut list_args = base_args(VcsEntityAction::List, VcsEntityResource::Repository);
    list_args.org_id = Some(org_id.to_owned());
    list_args.project_id = Some(project_id.to_owned());
    let list_result = server
        .vcs_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "repo list should succeed: {list_result:?}"
    );

    let body = result_json(&match list_result {
        Ok(r) => r,
        Err(e) => panic!("repo list response: {e}"),
    });
    let count = body.as_array().map_or(0, std::vec::Vec::len);
    assert!(
        count >= 2,
        "repo list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_vcs_repo_update() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-repo-upd";
    let project_id = "golden-vcs-proj-upd";
    let repo_id = "golden-vcs-repo-upd-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;

    let updated_payload = json!({
        "id": repo_id,
        "org_id": org_id,
        "project_id": project_id,
        "created_at": 0,
        "updated_at": 1,
        "name": "Updated Repo Name",
        "url": "https://github.com/test/updated.git",
        "local_path": "/tmp/updated",
        "vcs_type": "git"
    });

    let mut update_args = base_args(VcsEntityAction::Update, VcsEntityResource::Repository);
    update_args.org_id = Some(org_id.to_owned());
    update_args.project_id = Some(project_id.to_owned());
    update_args.data = Some(updated_payload);
    let update_result = server
        .vcs_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "repo update should succeed: {update_result:?}"
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Repository);
    get_args.id = Some(repo_id.to_owned());
    get_args.org_id = Some(org_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "repo get should succeed after update: {get_result:?}"
    );

    let body = result_json(&match get_result {
        Ok(r) => r,
        Err(e) => panic!("repo get after update response: {e}"),
    });
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some("Updated Repo Name")
    );
}

#[tokio::test]
async fn golden_vcs_repo_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-repo-del";
    let project_id = "golden-vcs-proj-del";
    let repo_id = "golden-vcs-repo-del-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;

    let mut delete_args = base_args(VcsEntityAction::Delete, VcsEntityResource::Repository);
    delete_args.id = Some(repo_id.to_owned());
    delete_args.org_id = Some(org_id.to_owned());
    delete_args.project_id = Some(project_id.to_owned());
    let delete_result = server
        .vcs_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "repo delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Repository);
    get_args.id = Some(repo_id.to_owned());
    get_args.org_id = Some(org_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "repo get should fail after delete");
}

// ---------------------------------------------------------------------------
// Branch CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_vcs_branch_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-branch-cg";
    let project_id = "golden-vcs-proj-bcg";
    let repo_id = "golden-vcs-repo-bcg";
    let branch_id = "golden-vcs-branch-cg-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let created = create_branch(&server, repo_id, branch_id, "feat/golden-branch").await;
    assert_eq!(
        created.get("id").and_then(serde_json::Value::as_str),
        Some(branch_id)
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Branch);
    get_args.id = Some(branch_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "branch get should succeed: {get_result:?}"
    );

    let body = result_json(&match get_result {
        Ok(r) => r,
        Err(e) => panic!("branch get response: {e}"),
    });
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(branch_id)
    );
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some("feat/golden-branch")
    );
}

#[tokio::test]
async fn golden_vcs_branch_list() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-branch-list";
    let project_id = "golden-vcs-proj-bl";
    let repo_id = "golden-vcs-repo-bl";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let _ = create_branch(&server, repo_id, "golden-vcs-branch-list-1", "main").await;
    let _ = create_branch(&server, repo_id, "golden-vcs-branch-list-2", "develop").await;

    let mut list_args = base_args(VcsEntityAction::List, VcsEntityResource::Branch);
    list_args.repository_id = Some(repo_id.to_owned());
    let list_result = server
        .vcs_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "branch list should succeed: {list_result:?}"
    );

    let body = result_json(&match list_result {
        Ok(r) => r,
        Err(e) => panic!("branch list response: {e}"),
    });
    let count = body.as_array().map_or(0, std::vec::Vec::len);
    assert!(
        count >= 2,
        "branch list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_vcs_branch_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-branch-del";
    let project_id = "golden-vcs-proj-bd";
    let repo_id = "golden-vcs-repo-bd";
    let branch_id = "golden-vcs-branch-del-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let _ = create_branch(&server, repo_id, branch_id, "feat/to-delete").await;

    let mut delete_args = base_args(VcsEntityAction::Delete, VcsEntityResource::Branch);
    delete_args.id = Some(branch_id.to_owned());
    let delete_result = server
        .vcs_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "branch delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Branch);
    get_args.id = Some(branch_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "branch get should fail after delete");
}

// ---------------------------------------------------------------------------
// Worktree CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn golden_vcs_worktree_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-wt-cg";
    let project_id = "golden-vcs-proj-wcg";
    let repo_id = "golden-vcs-repo-wcg";
    let branch_id = "golden-vcs-branch-wcg";
    let worktree_id = "golden-vcs-wt-cg-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let _ = create_branch(&server, repo_id, branch_id, "main").await;
    let created = create_worktree(&server, repo_id, branch_id, worktree_id).await;
    assert_eq!(
        created.get("id").and_then(serde_json::Value::as_str),
        Some(worktree_id)
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Worktree);
    get_args.id = Some(worktree_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "worktree get should succeed: {get_result:?}"
    );

    let body = result_json(&match get_result {
        Ok(r) => r,
        Err(e) => panic!("worktree get response: {e}"),
    });
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(worktree_id)
    );
    assert_eq!(
        body.get("repository_id")
            .and_then(serde_json::Value::as_str),
        Some(repo_id)
    );
}

#[tokio::test]
async fn golden_vcs_worktree_list() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-wt-list";
    let project_id = "golden-vcs-proj-wl";
    let repo_id = "golden-vcs-repo-wl";
    let branch_id = "golden-vcs-branch-wl";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let _ = create_branch(&server, repo_id, branch_id, "main").await;
    let _ = create_worktree(&server, repo_id, branch_id, "golden-vcs-wt-list-1").await;
    let _ = create_worktree(&server, repo_id, branch_id, "golden-vcs-wt-list-2").await;

    let mut list_args = base_args(VcsEntityAction::List, VcsEntityResource::Worktree);
    list_args.repository_id = Some(repo_id.to_owned());
    let list_result = server
        .vcs_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "worktree list should succeed: {list_result:?}"
    );

    let body = result_json(&match list_result {
        Ok(r) => r,
        Err(e) => panic!("worktree list response: {e}"),
    });
    let count = body.as_array().map_or(0, std::vec::Vec::len);
    assert!(
        count >= 2,
        "worktree list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_vcs_worktree_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-vcs-wt-del";
    let project_id = "golden-vcs-proj-wd";
    let repo_id = "golden-vcs-repo-wd";
    let branch_id = "golden-vcs-branch-wd";
    let worktree_id = "golden-vcs-wt-del-1";

    let _ = create_repo(&server, org_id, project_id, repo_id).await;
    let _ = create_branch(&server, repo_id, branch_id, "main").await;
    let _ = create_worktree(&server, repo_id, branch_id, worktree_id).await;

    let mut delete_args = base_args(VcsEntityAction::Delete, VcsEntityResource::Worktree);
    delete_args.id = Some(worktree_id.to_owned());
    let delete_result = server
        .vcs_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "worktree delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(VcsEntityAction::Get, VcsEntityResource::Worktree);
    get_args.id = Some(worktree_id.to_owned());
    let get_result = server
        .vcs_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "worktree get should fail after delete");
}
