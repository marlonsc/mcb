use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mcb_domain::entities::vcs::{RefDiff, VcsBranch, VcsCommit, VcsRepository};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::RepositoryId;
use mcb_server::tools::{
    ExecutionFlow, RuntimeDefaults, ToolExecutionContext, validate_execution_context,
};
use rstest::rstest;

struct TestVcsProvider {
    repo_root: PathBuf,
    repo_id: RepositoryId,
}

#[async_trait]
impl mcb_domain::ports::VcsProvider for TestVcsProvider {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        if path.starts_with(&self.repo_root) {
            return Ok(VcsRepository::new(
                self.repo_id,
                path.to_path_buf(),
                "main".to_owned(),
                vec!["main".to_owned()],
                None,
            ));
        }

        Err(Error::vcs("not a repository"))
    }

    fn repository_id(&self, _repo: &VcsRepository) -> RepositoryId {
        self.repo_id
    }

    async fn list_branches(&self, _repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
        Ok(Vec::new())
    }

    async fn commit_history(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        Ok(Vec::new())
    }

    async fn list_files(&self, _repo: &VcsRepository, _branch: &str) -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }

    async fn read_file(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _path: &Path,
    ) -> Result<String> {
        Ok(String::new())
    }

    fn vcs_name(&self) -> &str {
        "test"
    }

    async fn diff_refs(
        &self,
        _repo: &VcsRepository,
        _base_ref: &str,
        _head_ref: &str,
    ) -> Result<RefDiff> {
        Err(Error::vcs("not implemented"))
    }

    async fn list_repositories(&self, _root: &Path) -> Result<Vec<VcsRepository>> {
        Ok(Vec::new())
    }
}

fn valid_context() -> ToolExecutionContext {
    ToolExecutionContext {
        session_id: Some("session-1".to_owned()),
        parent_session_id: Some("parent-1".to_owned()),
        org_id: None,
        project_id: Some("project-1".to_owned()),
        worktree_id: Some("wt-1".to_owned()),
        repo_id: Some("repo-1".to_owned()),
        repo_path: Some("/tmp/repo".to_owned()),
        operator_id: Some("operator-1".to_owned()),
        machine_id: Some("machine-1".to_owned()),
        agent_program: Some("opencode".to_owned()),
        model_id: Some("gpt-5.3-codex".to_owned()),
        delegated: Some(false),
        timestamp: Some(1),
        execution_flow: Some(ExecutionFlow::StdioOnly.to_string()),
    }
}

#[rstest]
#[test]
fn rejects_blank_provenance_scope_for_search() {
    let mut context = valid_context();
    context.operator_id = Some("   ".to_owned());

    let validation = validate_execution_context("search", &context);
    assert!(validation.is_err(), "blank operator_id must be rejected");
    let error = match validation {
        Ok(()) => return,
        Err(error) => error,
    };
    assert!(error.message.contains("operator_id"));
}

#[rstest]
#[test]
fn rejects_delegated_without_parent_session_id() {
    let mut context = valid_context();
    context.delegated = Some(true);
    context.parent_session_id = Some(" ".to_owned());

    let validation = validate_execution_context("memory", &context);
    assert!(
        validation.is_err(),
        "delegated context must include parent_session_id"
    );
    let error = match validation {
        Ok(()) => return,
        Err(error) => error,
    };
    assert!(error.message.contains("parent_session_id"));
}

#[rstest]
#[test]
fn non_provenance_tool_bypasses_scope_gate() {
    let context = ToolExecutionContext {
        session_id: None,
        parent_session_id: None,
        org_id: None,
        project_id: None,
        worktree_id: None,
        repo_id: None,
        repo_path: None,
        operator_id: None,
        machine_id: None,
        agent_program: None,
        model_id: None,
        delegated: None,
        timestamp: None,
        execution_flow: Some(ExecutionFlow::StdioOnly.to_string()),
    };

    assert!(
        validate_execution_context("validate", &context).is_ok(),
        "non-index/search/memory tools should not require provenance scope"
    );
}

#[rstest]
#[test]
fn rejects_validate_in_server_hybrid_flow() {
    let mut context = valid_context();
    context.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());

    let validation = validate_execution_context("validate", &context);
    assert!(
        validation.is_err(),
        "validate must be rejected in server-hybrid flow"
    );
    let err = match validation {
        Ok(()) => return,
        Err(error) => error,
    };
    assert!(err.message.contains("Operation mode matrix violation"));
}

#[rstest]
#[test]
fn allows_search_in_client_hybrid_flow() {
    let mut context = valid_context();
    context.execution_flow = Some(ExecutionFlow::ClientHybrid.to_string());

    let validation = validate_execution_context("search", &context);
    assert!(
        validation.is_ok(),
        "search must be allowed in client-hybrid flow"
    );
}

#[rstest]
#[test]
fn allows_search_in_server_hybrid_flow() {
    let mut context = valid_context();
    context.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());

    assert!(
        validate_execution_context("search", &context).is_ok(),
        "search must be allowed in server-hybrid flow"
    );
}

#[rstest]
#[tokio::test]
async fn test_runtime_defaults_discover() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let repo_root = temp_dir.path().join("repo");
    let nested = repo_root.join("nested").join("path");
    std::fs::create_dir_all(&nested).expect("nested path");

    let provider = TestVcsProvider {
        repo_root: repo_root.clone(),
        repo_id: RepositoryId::from_name("repo-test"),
    };

    let defaults = RuntimeDefaults::discover_from_path(
        &provider,
        Some(nested.as_path()),
        Some(ExecutionFlow::StdioOnly),
    )
    .await;

    assert_eq!(defaults.workspace_root.as_deref(), repo_root.to_str());
    assert_eq!(defaults.repo_path.as_deref(), repo_root.to_str());
    assert_eq!(
        defaults.repo_id.as_deref(),
        Some(RepositoryId::from_name("repo-test").as_str().as_str())
    );
    assert_eq!(defaults.agent_program.as_deref(), Some("mcb-stdio"));
    assert_eq!(defaults.model_id.as_deref(), Some("unknown"));
    assert_eq!(defaults.execution_flow, Some(ExecutionFlow::StdioOnly));
    assert!(defaults.session_id.is_some());
}

#[rstest]
#[test]
fn test_resolve_overrides_beat_defaults() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("/defaults/workspace".to_owned()),
        repo_path: Some("/defaults/repo".to_owned()),
        repo_id: Some("repo-default".to_owned()),
        operator_id: Some("operator-default".to_owned()),
        machine_id: Some("machine-default".to_owned()),
        session_id: Some("session-default".to_owned()),
        agent_program: Some("mcb-stdio".to_owned()),
        model_id: Some("unknown".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
        client_session_id: None,
        org_id: None,
        project_id: None,
    };

    let overrides = HashMap::from([
        ("session_id".to_owned(), "session-override".to_owned()),
        ("repo_id".to_owned(), "repo-override".to_owned()),
        ("repo_path".to_owned(), "/repo/override".to_owned()),
        ("operator_id".to_owned(), "operator-override".to_owned()),
        ("machine_id".to_owned(), "machine-override".to_owned()),
        ("agent_program".to_owned(), "agent-override".to_owned()),
        ("model_id".to_owned(), "model-override".to_owned()),
        ("execution_flow".to_owned(), "client-hybrid".to_owned()),
        ("delegated".to_owned(), "true".to_owned()),
    ]);

    let context = ToolExecutionContext::resolve(&defaults, &overrides);

    assert_eq!(context.session_id.as_deref(), Some("session-override"));
    assert_eq!(context.repo_id.as_deref(), Some("repo-override"));
    assert_eq!(context.repo_path.as_deref(), Some("/repo/override"));
    assert_eq!(context.operator_id.as_deref(), Some("operator-override"));
    assert_eq!(context.machine_id.as_deref(), Some("machine-override"));
    assert_eq!(context.agent_program.as_deref(), Some("agent-override"));
    assert_eq!(context.model_id.as_deref(), Some("model-override"));
    assert_eq!(context.execution_flow.as_deref(), Some("client-hybrid"));
    assert_eq!(context.delegated, Some(true));
    assert!(context.timestamp.is_some());
}

#[rstest]
#[test]
fn test_resolve_with_empty_overrides_uses_defaults() {
    let defaults = RuntimeDefaults {
        workspace_root: Some("/defaults/workspace".to_owned()),
        repo_path: Some("/defaults/repo".to_owned()),
        repo_id: Some("repo-default".to_owned()),
        operator_id: Some("operator-default".to_owned()),
        machine_id: Some("machine-default".to_owned()),
        session_id: Some("session-default".to_owned()),
        agent_program: Some("mcb-stdio".to_owned()),
        model_id: Some("unknown".to_owned()),
        execution_flow: Some(ExecutionFlow::StdioOnly),
        client_session_id: None,
        org_id: None,
        project_id: None,
    };

    let context = ToolExecutionContext::resolve(&defaults, &HashMap::new());

    assert_eq!(context.session_id.as_deref(), Some("session-default"));
    assert_eq!(context.repo_id.as_deref(), Some("repo-default"));
    assert_eq!(context.repo_path.as_deref(), Some("/defaults/repo"));
    assert_eq!(context.operator_id.as_deref(), Some("operator-default"));
    assert_eq!(context.machine_id.as_deref(), Some("machine-default"));
    assert_eq!(context.agent_program.as_deref(), Some("mcb-stdio"));
    assert_eq!(context.model_id.as_deref(), Some("unknown"));
    assert_eq!(context.execution_flow.as_deref(), Some("stdio-only"));
}

#[rstest]
#[test]
fn test_resolve_workspace_root_maps_to_repo_path() {
    let defaults = RuntimeDefaults {
        workspace_root: None,
        repo_path: None,
        repo_id: None,
        operator_id: None,
        machine_id: None,
        session_id: None,
        agent_program: None,
        model_id: None,
        execution_flow: None,
        client_session_id: None,
        org_id: None,
        project_id: None,
    };
    let overrides = HashMap::from([(
        "workspace_root".to_owned(),
        "/workspace/override".to_owned(),
    )]);

    let context = ToolExecutionContext::resolve(&defaults, &overrides);
    assert_eq!(context.repo_path.as_deref(), Some("/workspace/override"));
}
