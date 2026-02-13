//!
//! Core MCP protocol server that orchestrates semantic code search operations.
//! Follows Clean Architecture principles with dependency injection.

use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use mcb_domain::constants::keys as schema;
use mcb_domain::ports::providers::VcsProvider;
use mcb_domain::ports::repositories::{
    IssueEntityRepository, OrgEntityRepository, PlanEntityRepository, ProjectRepository,
    VcsEntityRepository,
};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use mcb_domain::ports::services::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    ProjectDetectorService, SearchServiceInterface, ValidationServiceInterface,
};
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, Meta,
    PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use serde_json::Value;

use crate::handlers::{
    AgentHandler, EntityHandler, IndexHandler, IssueEntityHandler, MemoryHandler, OrgEntityHandler,
    PlanEntityHandler, ProjectHandler, SearchHandler, SessionHandler, ValidateHandler,
    VcsEntityHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
use crate::tools::{ToolExecutionContext, ToolHandlers, create_tool_list, route_tool_call};

fn meta_value_as_string(meta: &Meta, keys: &[&str]) -> Option<String> {
    for key in keys {
        let value = meta.get(*key)?;
        let extracted = match value {
            Value::String(v) => Some(v.clone()),
            Value::Number(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            _ => None,
        };
        if extracted.is_some() {
            return extracted;
        }
    }
    None
}

fn resolve_context_value(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<String> {
    request_meta
        .and_then(|meta| meta_value_as_string(meta, keys))
        .or_else(|| meta_value_as_string(context_meta, keys))
}

fn meta_value_as_bool(meta: &Meta, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let value = meta.get(*key)?;
        let extracted = match value {
            Value::Bool(v) => Some(*v),
            Value::String(v) => match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        };
        if extracted.is_some() {
            return extracted;
        }
    }
    None
}

fn resolve_context_bool(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<bool> {
    request_meta
        .and_then(|meta| meta_value_as_bool(meta, keys))
        .or_else(|| meta_value_as_bool(context_meta, keys))
}

/// Core MCP server implementation
///
/// This server implements the MCP protocol for semantic code search.
/// It depends only on domain services and receives all dependencies through
/// constructor injection following Clean Architecture principles.
#[derive(Clone)]
pub struct McpServer {
    /// Domain services for core operations
    services: McpServices,
    /// Tool handlers for MCP protocol
    handlers: ToolHandlers,
}

/// Domain services container (keeps struct field count manageable)
#[derive(Clone)]
pub struct McpServices {
    /// Indexing service
    pub indexing: Arc<dyn IndexingServiceInterface>,
    /// Context service
    pub context: Arc<dyn ContextServiceInterface>,
    /// Search service
    pub search: Arc<dyn SearchServiceInterface>,
    /// Validation service
    pub validation: Arc<dyn ValidationServiceInterface>,
    /// Memory service
    pub memory: Arc<dyn MemoryServiceInterface>,
    /// Agent session service
    pub agent_session: Arc<dyn AgentSessionServiceInterface>,
    /// Project detector service
    pub project: Arc<dyn ProjectDetectorService>,
    /// Project workflow repository
    pub project_workflow: Arc<dyn ProjectRepository>,
    /// VCS provider
    pub vcs: Arc<dyn VcsProvider>,
    /// VCS entity repository (repos, branches, worktrees)
    pub vcs_entity: Arc<dyn VcsEntityRepository>,
    /// Plan entity repository (plans, versions, reviews)
    pub plan_entity: Arc<dyn PlanEntityRepository>,
    /// Issue entity repository (issues, comments, labels, assignments)
    pub issue_entity: Arc<dyn IssueEntityRepository>,
    /// Org entity repository (orgs, users, teams, members, api keys)
    pub org_entity: Arc<dyn OrgEntityRepository>,
}

impl McpServer {
    async fn build_execution_context(
        &self,
        request: &CallToolRequestParams,
        context: &rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> ToolExecutionContext {
        let request_meta = request.meta.as_ref();
        let context_meta = &context.meta;

        let session_id = resolve_context_value(
            request_meta,
            context_meta,
            &["session_id", "sessionId", "x-session-id", "x_session_id"],
        );
        let parent_session_id = resolve_context_value(
            request_meta,
            context_meta,
            &[
                schema::PARENT_SESSION_ID,
                "parentSessionId",
                "x-parent-session-id",
                "x_parent_session_id",
            ],
        );
        let project_id = resolve_context_value(
            request_meta,
            context_meta,
            &[
                schema::PROJECT_ID,
                "projectId",
                "x-project-id",
                "x_project_id",
            ],
        );
        let worktree_id = resolve_context_value(
            request_meta,
            context_meta,
            &[
                schema::WORKTREE_ID,
                "worktreeId",
                "x-worktree-id",
                "x_worktree_id",
            ],
        );

        let mut repo_id = resolve_context_value(
            request_meta,
            context_meta,
            &[schema::REPO_ID, "repoId", "x-repo-id", "x_repo_id"],
        );
        let mut repo_path = resolve_context_value(
            request_meta,
            context_meta,
            &[schema::REPO_PATH, "repoPath", "x-repo-path", "x_repo_path"],
        );
        let operator_id = resolve_context_value(
            request_meta,
            context_meta,
            &[
                "operator_id",
                "operatorId",
                "x-operator-id",
                "x_operator_id",
            ],
        )
        .or_else(|| std::env::var("USER").ok());
        let machine_id = resolve_context_value(
            request_meta,
            context_meta,
            &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
        )
        .or_else(|| std::env::var("HOSTNAME").ok());
        let agent_program = resolve_context_value(
            request_meta,
            context_meta,
            &[
                "agent_program",
                "agentProgram",
                "ide",
                "x-agent-program",
                "x_agent_program",
            ],
        );
        let model_id = resolve_context_value(
            request_meta,
            context_meta,
            &["model_id", "model", "modelId", "x-model-id", "x_model_id"],
        );
        let delegated = resolve_context_bool(
            request_meta,
            context_meta,
            &["delegated", "is_delegated", "isDelegated", "x-delegated"],
        )
        .or(Some(parent_session_id.is_some()));

        if let Some(path_str) = repo_path.clone()
            && let Ok(repo) = self
                .services
                .vcs
                .open_repository(Path::new(&path_str))
                .await
        {
            repo_path = Some(repo.path().to_string_lossy().to_string());
            if repo_id.is_none() {
                repo_id = Some(self.services.vcs.repository_id(&repo).into_string());
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64);
        let execution_flow = std::env::var("MCB_EXECUTION_FLOW").ok();

        ToolExecutionContext {
            session_id,
            parent_session_id,
            project_id,
            worktree_id,
            repo_id,
            repo_path,
            operator_id,
            machine_id,
            agent_program,
            model_id,
            delegated,
            timestamp,
            execution_flow,
        }
    }

    /// Create a new MCP server with injected dependencies
    pub fn new(services: McpServices) -> Self {
        let hook_processor = HookProcessor::new(Some(services.memory.clone()));
        let vcs_entity_handler = Arc::new(VcsEntityHandler::new(services.vcs_entity.clone()));
        let plan_entity_handler = Arc::new(PlanEntityHandler::new(services.plan_entity.clone()));
        let issue_entity_handler = Arc::new(IssueEntityHandler::new(services.issue_entity.clone()));
        let org_entity_handler = Arc::new(OrgEntityHandler::new(services.org_entity.clone()));
        let entity_handler = Arc::new(EntityHandler::new(
            Arc::clone(&vcs_entity_handler),
            Arc::clone(&plan_entity_handler),
            Arc::clone(&issue_entity_handler),
            Arc::clone(&org_entity_handler),
        ));

        let handlers = ToolHandlers {
            index: Arc::new(IndexHandler::new(services.indexing.clone())),
            search: Arc::new(SearchHandler::new(
                services.search.clone(),
                services.memory.clone(),
            )),
            validate: Arc::new(ValidateHandler::new(services.validation.clone())),
            memory: Arc::new(MemoryHandler::new(services.memory.clone())),
            session: Arc::new(SessionHandler::new(
                services.agent_session.clone(),
                services.memory.clone(),
            )),
            agent: Arc::new(AgentHandler::new(services.agent_session.clone())),
            project: Arc::new(ProjectHandler::new(services.project_workflow.clone())),
            vcs: Arc::new(VcsHandler::new(services.vcs.clone())),
            vcs_entity: vcs_entity_handler,
            plan_entity: plan_entity_handler,
            issue_entity: issue_entity_handler,
            org_entity: org_entity_handler,
            entity: entity_handler,
            hook_processor: Arc::new(hook_processor),
        };

        Self { services, handlers }
    }

    /// Create a new MCP server from domain services
    /// This is the preferred constructor that uses the DI container
    pub fn from_services(services: McpServices) -> Self {
        Self::new(services)
    }

    /// Access to indexing service
    pub fn indexing_service(&self) -> Arc<dyn IndexingServiceInterface> {
        Arc::clone(&self.services.indexing)
    }

    /// Access to context service
    pub fn context_service(&self) -> Arc<dyn ContextServiceInterface> {
        Arc::clone(&self.services.context)
    }

    /// Access to VCS provider (for branch/repo handlers)
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        Arc::clone(&self.services.vcs)
    }

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.services.search)
    }

    /// Access to validation service
    pub fn validation_service(&self) -> Arc<dyn ValidationServiceInterface> {
        Arc::clone(&self.services.validation)
    }

    /// Access to memory service
    pub fn memory_service(&self) -> Arc<dyn MemoryServiceInterface> {
        Arc::clone(&self.services.memory)
    }

    /// Access to agent session service
    pub fn agent_session_service(&self) -> Arc<dyn AgentSessionServiceInterface> {
        Arc::clone(&self.services.agent_session)
    }

    /// Access to project service
    pub fn project_service(&self) -> Arc<dyn ProjectDetectorService> {
        Arc::clone(&self.services.project)
    }

    /// Access to project workflow repository.
    pub fn project_workflow_repository(&self) -> Arc<dyn ProjectRepository> {
        Arc::clone(&self.services.project_workflow)
    }

    /// Access to VCS entity repository.
    pub fn vcs_entity_repository(&self) -> Arc<dyn VcsEntityRepository> {
        Arc::clone(&self.services.vcs_entity)
    }

    /// Access to plan entity repository.
    pub fn plan_entity_repository(&self) -> Arc<dyn PlanEntityRepository> {
        Arc::clone(&self.services.plan_entity)
    }

    /// Access to issue entity repository.
    pub fn issue_entity_repository(&self) -> Arc<dyn IssueEntityRepository> {
        Arc::clone(&self.services.issue_entity)
    }

    /// Access to organization entity repository.
    pub fn org_entity_repository(&self) -> Arc<dyn OrgEntityRepository> {
        Arc::clone(&self.services.org_entity)
    }

    /// Access to index handler (for HTTP transport)
    pub fn index_handler(&self) -> Arc<IndexHandler> {
        Arc::clone(&self.handlers.index)
    }

    /// Access to search handler (for HTTP transport)
    pub fn search_handler(&self) -> Arc<SearchHandler> {
        Arc::clone(&self.handlers.search)
    }

    /// Access to validate handler (for HTTP transport)
    pub fn validate_handler(&self) -> Arc<ValidateHandler> {
        Arc::clone(&self.handlers.validate)
    }

    /// Access to memory handler (for HTTP transport)
    pub fn memory_handler(&self) -> Arc<MemoryHandler> {
        Arc::clone(&self.handlers.memory)
    }

    /// Access to session handler (for HTTP transport)
    pub fn session_handler(&self) -> Arc<SessionHandler> {
        Arc::clone(&self.handlers.session)
    }

    /// Access to agent handler (for HTTP transport)
    pub fn agent_handler(&self) -> Arc<AgentHandler> {
        Arc::clone(&self.handlers.agent)
    }

    /// Access to VCS handler (for HTTP transport)
    pub fn vcs_handler(&self) -> Arc<VcsHandler> {
        Arc::clone(&self.handlers.vcs)
    }

    /// Access to unified entity handler (for HTTP transport)
    pub fn entity_handler(&self) -> Arc<EntityHandler> {
        Arc::clone(&self.handlers.entity)
    }

    /// Access to project handler (for HTTP transport)
    pub fn project_handler(&self) -> Arc<ProjectHandler> {
        Arc::clone(&self.handlers.project)
    }

    /// Access to VCS entity handler (for HTTP transport)
    pub fn vcs_entity_handler(&self) -> Arc<VcsEntityHandler> {
        Arc::clone(&self.handlers.vcs_entity)
    }

    /// Access to plan entity handler (for HTTP transport)
    pub fn plan_entity_handler(&self) -> Arc<PlanEntityHandler> {
        Arc::clone(&self.handlers.plan_entity)
    }

    /// Access to issue entity handler (for HTTP transport)
    pub fn issue_entity_handler(&self) -> Arc<IssueEntityHandler> {
        Arc::clone(&self.handlers.issue_entity)
    }

    /// Access to org entity handler (for HTTP transport)
    pub fn org_entity_handler(&self) -> Arc<OrgEntityHandler> {
        Arc::clone(&self.handlers.org_entity)
    }

    /// Access to hook processor (for automatic memory operations)
    pub fn hook_processor(&self) -> Arc<HookProcessor> {
        Arc::clone(&self.handlers.hook_processor)
    }

    /// Clone the complete tool handlers set for unified internal execution.
    pub fn tool_handlers(&self) -> ToolHandlers {
        self.handlers.clone()
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                r#"MCP Context Browser - Semantic Code Search

tools:
- index: Index operations (start, status, clear)
- search: Unified search for code or memory
- validate: Validation and analysis operations
- memory: Memory storage, retrieval, timeline, inject
- session: Agent session lifecycle + summaries
- agent: Agent activity logging
- project: Project workflow management
- vcs: Repository operations
- entity: Unified entity CRUD (vcs/plan/issue/org resources)
"#
                .to_string(),
            ),
        }
    }

    /// List available tools
    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = create_tool_list()?;
        Ok(ListToolsResult {
            tools,
            meta: Default::default(),
            next_cursor: None,
        })
    }

    /// Call a tool
    async fn call_tool(
        &self,
        mut request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let execution_context = self.build_execution_context(&request, &context).await;
        execution_context.apply_to_request_if_missing(&mut request);

        route_tool_call(request, &self.handlers, execution_context).await
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_context_value;
    use crate::tools::ToolExecutionContext;
    use rmcp::model::{CallToolRequestParams, Meta};
    use serde_json::Value;

    #[test]
    fn resolve_context_value_prefers_request_meta() {
        let mut request_meta = Meta::new();
        request_meta.insert(
            "session_id".to_string(),
            Value::String("req-session".to_string()),
        );

        let mut context_meta = Meta::new();
        context_meta.insert(
            "session_id".to_string(),
            Value::String("ctx-session".to_string()),
        );

        let resolved = resolve_context_value(Some(&request_meta), &context_meta, &["session_id"]);
        assert_eq!(resolved.as_deref(), Some("req-session"));
    }

    #[test]
    fn resolve_context_value_uses_context_meta_when_request_missing() {
        let request_meta = Meta::new();

        let mut context_meta = Meta::new();
        context_meta.insert(
            "sessionId".to_string(),
            Value::String("ctx-session".to_string()),
        );

        let resolved = resolve_context_value(Some(&request_meta), &context_meta, &["sessionId"]);
        assert_eq!(resolved.as_deref(), Some("ctx-session"));
    }

    #[test]
    fn execution_context_injection_does_not_override_existing_value() {
        let mut request = CallToolRequestParams {
            meta: None,
            name: "memory".into(),
            arguments: Some(Default::default()),
            task: None,
        };

        request.arguments.as_mut().expect("arguments").insert(
            "session_id".to_string(),
            Value::String("existing".to_string()),
        );

        ToolExecutionContext {
            session_id: Some("from-meta".to_string()),
            timestamp: Some(1234567890),
            ..Default::default()
        }
        .apply_to_request_if_missing(&mut request);

        let value = request
            .arguments
            .and_then(|args| args.get("session_id").cloned())
            .expect("session_id argument");
        assert_eq!(value, Value::String("existing".to_string()));
    }
}
