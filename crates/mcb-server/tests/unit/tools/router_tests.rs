//! Execution validation: provenance gates, operation mode matrix, boot discovery.
//!
//! Tests verify that the server enforces provenance scope (session, repo, operator)
//! for data-plane tools and respects the operation mode matrix for flow restrictions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mcb_domain::entities::vcs::{RefDiff, VcsBranch, VcsCommit, VcsRepository};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::RepositoryId;
use mcb_server::tools::{
    ExecutionFlow, RuntimeDefaults, ToolExecutionContext, validate_execution_context,
};
use mcb_utils::constants::FALLBACK_UNKNOWN;
use mcb_utils::constants::ide::KNOWN_IDE_PROGRAMS;
use rstest::{fixture, rstest};

// ─── Test VCS provider ───────────────────────────────────────────────

struct StubVcs {
    repo_root: PathBuf,
    repo_id: RepositoryId,
}

#[async_trait]
impl mcb_domain::ports::VcsProvider for StubVcs {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        if path.starts_with(&self.repo_root) {
            Ok(VcsRepository::new(
                self.repo_id,
                path.to_path_buf(),
                "main".to_owned(),
                vec!["main".to_owned()],
                None,
            ))
        } else {
            Err(Error::vcs("not a repository"))
        }
    }
    fn repository_id(&self, _: &VcsRepository) -> RepositoryId {
        self.repo_id
    }
    async fn list_branches(&self, _: &VcsRepository) -> Result<Vec<VcsBranch>> {
        Ok(vec![])
    }
    async fn commit_history(
        &self,
        _: &VcsRepository,
        _: &str,
        _: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        Ok(vec![])
    }
    async fn list_files(&self, _: &VcsRepository, _: &str) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }
    async fn read_file(&self, _: &VcsRepository, _: &str, _: &Path) -> Result<String> {
        Ok(String::new())
    }
    fn vcs_name(&self) -> &str {
        "test"
    }
    async fn diff_refs(&self, _: &VcsRepository, _: &str, _: &str) -> Result<RefDiff> {
        Err(Error::vcs("not implemented"))
    }
    async fn list_repositories(&self, _: &Path) -> Result<Vec<VcsRepository>> {
        Ok(vec![])
    }
}

// ─── Fixtures ────────────────────────────────────────────────────────

/// Fully-populated context that passes all validation gates.
#[fixture]
fn valid_ctx() -> ToolExecutionContext {
    ToolExecutionContext {
        session_id: Some("s1".to_owned()),
        parent_session_id: Some("p1".to_owned()),
        org_id: None,
        project_id: Some("proj1".to_owned()),
        worktree_id: Some("wt1".to_owned()),
        repo_id: Some("r1".to_owned()),
        repo_path: Some("/tmp/repo".to_owned()),
        operator_id: Some("dev".to_owned()),
        machine_id: Some("laptop".to_owned()),
        agent_program: Some("opencode".to_owned()),
        model_id: Some("gpt-5".to_owned()),
        delegated: Some(false),
        timestamp: Some(1),
        execution_flow: Some(ExecutionFlow::StdioOnly.to_string()),
    }
}

// ─── Provenance scope gates ──────────────────────────────────────────

#[rstest]
fn blank_operator_rejected_for_data_plane_tools(mut valid_ctx: ToolExecutionContext) {
    valid_ctx.operator_id = Some("   ".to_owned());

    let err = validate_execution_context("search_code", &valid_ctx).unwrap_err();
    assert!(err.message.contains("operator_id"));
}

#[rstest]
fn delegated_agent_requires_parent_session(mut valid_ctx: ToolExecutionContext) {
    valid_ctx.delegated = Some(true);
    valid_ctx.parent_session_id = Some(" ".to_owned());

    let err = validate_execution_context("store_memory", &valid_ctx).unwrap_err();
    assert!(err.message.contains("parent_session_id"));
}

#[rstest]
fn non_data_plane_tools_skip_provenance_check() {
    let empty = ToolExecutionContext {
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

    assert!(validate_execution_context("validate_code", &empty).is_ok());
}

// ─── Operation mode matrix ───────────────────────────────────────────

#[rstest]
fn validate_rejected_in_server_hybrid_flow(mut valid_ctx: ToolExecutionContext) {
    valid_ctx.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());

    let err = validate_execution_context("validate_code", &valid_ctx).unwrap_err();
    assert!(err.message.contains("Operation mode matrix violation"));
}

#[rstest]
fn search_allowed_in_client_hybrid_flow(mut valid_ctx: ToolExecutionContext) {
    valid_ctx.execution_flow = Some(ExecutionFlow::ClientHybrid.to_string());
    assert!(validate_execution_context("search_code", &valid_ctx).is_ok());
}

#[rstest]
fn search_allowed_in_server_hybrid_flow(mut valid_ctx: ToolExecutionContext) {
    valid_ctx.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());
    assert!(validate_execution_context("search_code", &valid_ctx).is_ok());
}

// ─── Boot-time workspace discovery ──────────────────────────────────

#[rstest]
#[tokio::test]
async fn boot_discovers_workspace_from_nested_path() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let root = tmp.path().join("repo");
    let nested = root.join("src/deep/path");
    std::fs::create_dir_all(&nested).expect("dirs");

    let vcs = StubVcs {
        repo_root: root.clone(),
        repo_id: RepositoryId::from_name("test-repo"),
    };

    let defaults = RuntimeDefaults::discover_from_path(
        &vcs,
        Some(nested.as_path()),
        Some(ExecutionFlow::StdioOnly),
    )
    .await;

    assert_eq!(defaults.workspace_root.as_deref(), root.to_str());
    assert_eq!(defaults.repo_path.as_deref(), root.to_str());
    assert!(defaults.repo_id.is_some());
    assert!(defaults.session_id.is_some());
    assert_eq!(defaults.model_id.as_deref(), Some(FALLBACK_UNKNOWN));
    assert_eq!(defaults.execution_flow, Some(ExecutionFlow::StdioOnly));
    assert!(
        defaults
            .agent_program
            .as_deref()
            .is_some_and(|p| KNOWN_IDE_PROGRAMS.contains(&p)),
        "expected known IDE, got {:?}",
        defaults.agent_program
    );
}

#[rstest]
fn workspace_root_override_sets_repo_path() {
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
    let overrides = HashMap::from([("workspace_root".to_owned(), "/ws".to_owned())]);

    let ctx = ToolExecutionContext::resolve(&defaults, &overrides);
    assert_eq!(ctx.repo_path.as_deref(), Some("/ws"));
}
