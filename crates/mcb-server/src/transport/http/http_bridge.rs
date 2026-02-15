use rocket::Request;
use rocket::request::{FromRequest, Outcome};

#[derive(Debug, Clone)]
pub(super) struct BridgeProvenance {
    pub(super) workspace_root: Option<String>,
    pub(super) repo_path: Option<String>,
    pub(super) repo_id: Option<String>,
    pub(super) session_id: Option<String>,
    pub(super) parent_session_id: Option<String>,
    pub(super) project_id: Option<String>,
    pub(super) worktree_id: Option<String>,
    pub(super) operator_id: Option<String>,
    pub(super) machine_id: Option<String>,
    pub(super) agent_program: Option<String>,
    pub(super) model_id: Option<String>,
    pub(super) delegated: Option<String>,
    pub(super) execution_flow: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BridgeProvenance {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let header = |name: &str| request.headers().get_one(name).map(ToOwned::to_owned);

        Outcome::Success(Self {
            workspace_root: header("X-Workspace-Root"),
            repo_path: header("X-Repo-Path"),
            repo_id: header("X-Repo-Id"),
            session_id: header("X-Session-Id"),
            parent_session_id: header("X-Parent-Session-Id"),
            project_id: header("X-Project-Id"),
            worktree_id: header("X-Worktree-Id"),
            operator_id: header("X-Operator-Id"),
            machine_id: header("X-Machine-Id"),
            agent_program: header("X-Agent-Program"),
            model_id: header("X-Model-Id"),
            delegated: header("X-Delegated"),
            execution_flow: header("X-Execution-Flow"),
        })
    }
}
