//! Tool Router Module
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use std::sync::Arc;

use crate::args::{
    AnalyzeComplexityArgs, AnalyzeImpactArgs, ClearIndexArgs, CompareBranchesArgs,
    CreateAgentSessionArgs, CreateSessionSummaryArgs, GetAgentSessionArgs, GetIndexingStatusArgs,
    GetSessionSummaryArgs, GetValidationRulesArgs, IndexCodebaseArgs, IndexVcsRepositoryArgs,
    ListAgentSessionsArgs, ListRepositoriesArgs, ListValidatorsArgs, MemoryGetErrorPatternsArgs,
    MemoryGetExecutionsArgs, MemoryGetObservationsArgs, MemoryGetQualityGatesArgs,
    MemoryInjectContextArgs, MemoryRecordErrorPatternArgs, MemorySearchArgs,
    MemoryStoreExecutionArgs, MemoryStoreQualityGateArgs, MemoryTimelineArgs,
    ProjectAddDependencyArgs, ProjectCreateIssueArgs, ProjectCreatePhaseArgs,
    ProjectListDecisionsArgs, ProjectListIssuesArgs, ProjectListPhasesArgs,
    ProjectRecordDecisionArgs, ProjectUpdateIssueArgs, ProjectUpdatePhaseArgs, SearchBranchArgs,
    SearchCodeArgs, SearchMemoriesArgs, StoreDelegationArgs, StoreObservationArgs,
    StoreToolCallArgs, UpdateAgentSessionArgs, ValidateArchitectureArgs, ValidateFileArgs,
};
use crate::handlers::{
    AnalyzeComplexityHandler, AnalyzeImpactHandler, ClearIndexHandler, CompareBranchesHandler,
    CreateAgentSessionHandler, CreateSessionSummaryHandler, GetAgentSessionHandler,
    GetExecutionsHandler, GetIndexingStatusHandler, GetQualityGatesHandler,
    GetSessionSummaryHandler, GetValidationRulesHandler, IndexCodebaseHandler,
    IndexVcsRepositoryHandler, ListAgentSessionsHandler, ListRepositoriesHandler,
    ListValidatorsHandler, MemoryGetErrorPatternsHandler, MemoryGetObservationsHandler,
    MemoryInjectContextHandler, MemoryRecordErrorPatternHandler, MemorySearchHandler,
    MemoryTimelineHandler, ProjectAddDependencyHandler, ProjectCreateIssueHandler,
    ProjectCreatePhaseHandler, ProjectListDecisionsHandler, ProjectListIssuesHandler,
    ProjectListPhasesHandler, ProjectRecordDecisionHandler, ProjectUpdateIssueHandler,
    ProjectUpdatePhaseHandler, SearchBranchHandler, SearchCodeHandler, SearchMemoriesHandler,
    StoreDelegationHandler, StoreExecutionHandler, StoreObservationHandler,
    StoreQualityGateHandler, StoreToolCallHandler, UpdateAgentSessionHandler,
    ValidateArchitectureHandler, ValidateFileHandler,
};

/// Handler references for tool routing
#[derive(Clone)]
pub struct ToolHandlers {
    /// Handler for codebase indexing operations
    pub index_codebase: Arc<IndexCodebaseHandler>,
    /// Handler for code search operations
    pub search_code: Arc<SearchCodeHandler>,
    /// Handler for indexing status operations
    pub get_indexing_status: Arc<GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    pub clear_index: Arc<ClearIndexHandler>,
    /// Handler for architecture validation operations
    pub validate_architecture: Arc<ValidateArchitectureHandler>,
    /// Handler for single file validation
    pub validate_file: Arc<ValidateFileHandler>,
    /// Handler for listing validators
    pub list_validators: Arc<ListValidatorsHandler>,
    /// Handler for getting validation rules
    pub get_validation_rules: Arc<GetValidationRulesHandler>,
    /// Handler for complexity analysis
    pub analyze_complexity: Arc<AnalyzeComplexityHandler>,
    /// Handler for VCS repository indexing
    pub index_vcs_repository: Arc<IndexVcsRepositoryHandler>,
    /// Handler for branch-specific search
    pub search_branch: Arc<SearchBranchHandler>,
    /// Handler for listing indexed repositories
    pub list_repositories: Arc<ListRepositoriesHandler>,
    /// Handler for comparing branches
    pub compare_branches: Arc<CompareBranchesHandler>,
    /// Handler for impact analysis
    pub analyze_impact: Arc<AnalyzeImpactHandler>,
    /// Handler for storing observations in memory
    pub store_observation: Arc<StoreObservationHandler>,
    /// Handler for searching memories
    pub search_memories: Arc<SearchMemoriesHandler>,
    /// Handler for getting session summaries
    pub get_session_summary: Arc<GetSessionSummaryHandler>,
    /// Handler for creating session summaries
    pub create_session_summary: Arc<CreateSessionSummaryHandler>,
    /// Handler for memory timeline operations (MEM-04)
    pub memory_timeline: Arc<MemoryTimelineHandler>,
    /// Handler for getting observation details (MEM-04)
    pub memory_get_observations: Arc<MemoryGetObservationsHandler>,
    /// Handler for context injection (MEM-08)
    pub memory_inject_context: Arc<MemoryInjectContextHandler>,
    /// Handler for token-efficient memory search (Step 1 - MEM-04a)
    pub memory_search: Arc<MemorySearchHandler>,
    /// Handler for storing execution results
    pub memory_store_execution: Arc<StoreExecutionHandler>,
    /// Handler for retrieving execution history
    pub memory_get_executions: Arc<GetExecutionsHandler>,
    /// Handler for storing quality gate results
    pub memory_store_quality_gate: Arc<StoreQualityGateHandler>,
    /// Handler for retrieving quality gate results
    pub memory_get_quality_gates: Arc<GetQualityGatesHandler>,
    /// Handler for recording error patterns
    pub memory_record_error_pattern: Arc<MemoryRecordErrorPatternHandler>,
    /// Handler for retrieving error patterns
    pub memory_get_error_patterns: Arc<MemoryGetErrorPatternsHandler>,
    /// Handler for creating agent sessions
    pub create_agent_session: Arc<CreateAgentSessionHandler>,
    /// Handler for getting agent session details
    pub get_agent_session: Arc<GetAgentSessionHandler>,
    /// Handler for updating agent sessions
    pub update_agent_session: Arc<UpdateAgentSessionHandler>,
    /// Handler for listing agent sessions
    pub list_agent_sessions: Arc<ListAgentSessionsHandler>,
    /// Handler for storing tool calls
    pub store_tool_call: Arc<StoreToolCallHandler>,
    /// Handler for storing delegations
    pub store_delegation: Arc<StoreDelegationHandler>,
    /// Handler for creating project phases
    pub project_create_phase: Arc<ProjectCreatePhaseHandler>,
    /// Handler for updating project phases
    pub project_update_phase: Arc<ProjectUpdatePhaseHandler>,
    /// Handler for listing project phases
    pub project_list_phases: Arc<ProjectListPhasesHandler>,
    /// Handler for creating project issues
    pub project_create_issue: Arc<ProjectCreateIssueHandler>,
    /// Handler for updating project issues
    pub project_update_issue: Arc<ProjectUpdateIssueHandler>,
    /// Handler for listing project issues
    pub project_list_issues: Arc<ProjectListIssuesHandler>,
    /// Handler for adding issue dependencies
    pub project_add_dependency: Arc<ProjectAddDependencyHandler>,
    /// Handler for recording project decisions
    pub project_record_decision: Arc<ProjectRecordDecisionHandler>,
    /// Handler for listing project decisions
    pub project_list_decisions: Arc<ProjectListDecisionsHandler>,
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
pub async fn route_tool_call(
    request: CallToolRequestParams,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "index_codebase" => {
            let args = parse_args::<IndexCodebaseArgs>(&request)?;
            handlers.index_codebase.handle(Parameters(args)).await
        }
        "search_code" => {
            let args = parse_args::<SearchCodeArgs>(&request)?;
            handlers.search_code.handle(Parameters(args)).await
        }
        "get_indexing_status" => {
            let args = parse_args::<GetIndexingStatusArgs>(&request)?;
            handlers.get_indexing_status.handle(Parameters(args)).await
        }
        "clear_index" => {
            let args = parse_args::<ClearIndexArgs>(&request)?;
            handlers.clear_index.handle(Parameters(args)).await
        }
        "validate_architecture" => {
            let args = parse_args::<ValidateArchitectureArgs>(&request)?;
            handlers
                .validate_architecture
                .handle(Parameters(args))
                .await
        }
        "validate_file" => {
            let args = parse_args::<ValidateFileArgs>(&request)?;
            handlers.validate_file.handle(Parameters(args)).await
        }
        "list_validators" => {
            let args = parse_args::<ListValidatorsArgs>(&request)?;
            handlers.list_validators.handle(Parameters(args)).await
        }
        "get_validation_rules" => {
            let args = parse_args::<GetValidationRulesArgs>(&request)?;
            handlers.get_validation_rules.handle(Parameters(args)).await
        }
        "analyze_complexity" => {
            let args = parse_args::<AnalyzeComplexityArgs>(&request)?;
            handlers.analyze_complexity.handle(Parameters(args)).await
        }
        "index_vcs_repository" => {
            let args = parse_args::<IndexVcsRepositoryArgs>(&request)?;
            handlers.index_vcs_repository.handle(Parameters(args)).await
        }
        "search_branch" => {
            let args = parse_args::<SearchBranchArgs>(&request)?;
            handlers.search_branch.handle(Parameters(args)).await
        }
        "list_repositories" => {
            let args = parse_args::<ListRepositoriesArgs>(&request)?;
            handlers.list_repositories.handle(Parameters(args)).await
        }
        "compare_branches" => {
            let args = parse_args::<CompareBranchesArgs>(&request)?;
            handlers.compare_branches.handle(Parameters(args)).await
        }
        "analyze_impact" => {
            let args = parse_args::<AnalyzeImpactArgs>(&request)?;
            handlers.analyze_impact.handle(Parameters(args)).await
        }
        "store_observation" => {
            let args = parse_args::<StoreObservationArgs>(&request)?;
            handlers.store_observation.handle(Parameters(args)).await
        }
        "search_memories" => {
            let args = parse_args::<SearchMemoriesArgs>(&request)?;
            handlers.search_memories.handle(Parameters(args)).await
        }
        "get_session_summary" => {
            let args = parse_args::<GetSessionSummaryArgs>(&request)?;
            handlers.get_session_summary.handle(Parameters(args)).await
        }
        "create_session_summary" => {
            let args = parse_args::<CreateSessionSummaryArgs>(&request)?;
            handlers
                .create_session_summary
                .handle(Parameters(args))
                .await
        }
        "memory_timeline" => {
            let args = parse_args::<MemoryTimelineArgs>(&request)?;
            handlers.memory_timeline.handle(Parameters(args)).await
        }
        "memory_get_observations" => {
            let args = parse_args::<MemoryGetObservationsArgs>(&request)?;
            handlers
                .memory_get_observations
                .handle(Parameters(args))
                .await
        }
        "memory_inject_context" => {
            let args = parse_args::<MemoryInjectContextArgs>(&request)?;
            handlers
                .memory_inject_context
                .handle(Parameters(args))
                .await
        }
        "memory_search" => {
            let args = parse_args::<MemorySearchArgs>(&request)?;
            handlers.memory_search.handle(Parameters(args)).await
        }
        "memory_store_execution" => {
            let args = parse_args::<MemoryStoreExecutionArgs>(&request)?;
            handlers
                .memory_store_execution
                .handle(Parameters(args))
                .await
        }
        "memory_get_executions" => {
            let args = parse_args::<MemoryGetExecutionsArgs>(&request)?;
            handlers
                .memory_get_executions
                .handle(Parameters(args))
                .await
        }
        "memory_store_quality_gate" => {
            let args = parse_args::<MemoryStoreQualityGateArgs>(&request)?;
            handlers
                .memory_store_quality_gate
                .handle(Parameters(args))
                .await
        }
        "memory_get_quality_gates" => {
            let args = parse_args::<MemoryGetQualityGatesArgs>(&request)?;
            handlers
                .memory_get_quality_gates
                .handle(Parameters(args))
                .await
        }
        "memory_record_error_pattern" => {
            let args = parse_args::<MemoryRecordErrorPatternArgs>(&request)?;
            handlers
                .memory_record_error_pattern
                .handle(Parameters(args))
                .await
        }
        "memory_get_error_patterns" => {
            let args = parse_args::<MemoryGetErrorPatternsArgs>(&request)?;
            handlers
                .memory_get_error_patterns
                .handle(Parameters(args))
                .await
        }
        "create_agent_session" => {
            let args = parse_args::<CreateAgentSessionArgs>(&request)?;
            handlers.create_agent_session.handle(Parameters(args)).await
        }
        "get_agent_session" => {
            let args = parse_args::<GetAgentSessionArgs>(&request)?;
            handlers.get_agent_session.handle(Parameters(args)).await
        }
        "update_agent_session" => {
            let args = parse_args::<UpdateAgentSessionArgs>(&request)?;
            handlers.update_agent_session.handle(Parameters(args)).await
        }
        "list_agent_sessions" => {
            let args = parse_args::<ListAgentSessionsArgs>(&request)?;
            handlers.list_agent_sessions.handle(Parameters(args)).await
        }
        "store_tool_call" => {
            let args = parse_args::<StoreToolCallArgs>(&request)?;
            handlers.store_tool_call.handle(Parameters(args)).await
        }
        "store_delegation" => {
            let args = parse_args::<StoreDelegationArgs>(&request)?;
            handlers.store_delegation.handle(Parameters(args)).await
        }
        "project_create_phase" => {
            let args = parse_args::<ProjectCreatePhaseArgs>(&request)?;
            handlers.project_create_phase.handle(Parameters(args)).await
        }
        "project_update_phase" => {
            let args = parse_args::<ProjectUpdatePhaseArgs>(&request)?;
            handlers.project_update_phase.handle(Parameters(args)).await
        }
        "project_list_phases" => {
            let args = parse_args::<ProjectListPhasesArgs>(&request)?;
            handlers.project_list_phases.handle(Parameters(args)).await
        }
        "project_create_issue" => {
            let args = parse_args::<ProjectCreateIssueArgs>(&request)?;
            handlers.project_create_issue.handle(Parameters(args)).await
        }
        "project_update_issue" => {
            let args = parse_args::<ProjectUpdateIssueArgs>(&request)?;
            handlers.project_update_issue.handle(Parameters(args)).await
        }
        "project_list_issues" => {
            let args = parse_args::<ProjectListIssuesArgs>(&request)?;
            handlers.project_list_issues.handle(Parameters(args)).await
        }
        "project_add_dependency" => {
            let args = parse_args::<ProjectAddDependencyArgs>(&request)?;
            handlers
                .project_add_dependency
                .handle(Parameters(args))
                .await
        }
        "project_record_decision" => {
            let args = parse_args::<ProjectRecordDecisionArgs>(&request)?;
            handlers
                .project_record_decision
                .handle(Parameters(args))
                .await
        }
        "project_list_decisions" => {
            let args = parse_args::<ProjectListDecisionsArgs>(&request)?;
            handlers
                .project_list_decisions
                .handle(Parameters(args))
                .await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }
}

/// Parse request arguments into the expected type
fn parse_args<T: serde::de::DeserializeOwned>(
    request: &CallToolRequestParams,
) -> Result<T, McpError> {
    let args_value = serde_json::Value::Object(request.arguments.clone().unwrap_or_default());
    serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))
}
