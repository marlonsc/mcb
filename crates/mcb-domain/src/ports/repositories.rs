//! Repository ports for data persistence.
//!
//! All repository traits use `#[async_trait]` and require `Send + Sync`.

use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;

use crate::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, Delegation, ToolCall,
};
use crate::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use crate::entities::memory::{MemoryFilter, Observation, SessionSummary};
use crate::entities::plan::{Plan, PlanReview, PlanVersion};
use crate::entities::project::Project;
use crate::entities::repository::{Branch, Repository};
use crate::entities::worktree::{AgentWorktreeAssignment, Worktree};
use crate::entities::{ApiKey, Organization, Team, TeamMember, Transition, User};
use crate::entities::{WorkflowSession, WorkflowState};
use crate::error::Result;
use crate::ports::admin::IndexingOperation;
use crate::value_objects::ids::{ObservationId, SessionId};
use crate::value_objects::{CollectionId, OperationId};

// ============================================================================
// Organization
// ============================================================================

/// Registry for organizations.
#[async_trait]
pub trait OrgRegistry: Send + Sync {
    /// Create an organization.
    async fn create_org(&self, org: &Organization) -> Result<()>;
    /// Get an organization by ID.
    async fn get_org(&self, id: &str) -> Result<Organization>;
    /// List all organizations.
    async fn list_orgs(&self) -> Result<Vec<Organization>>;
    /// Update an organization.
    async fn update_org(&self, org: &Organization) -> Result<()>;
    /// Delete an organization by ID.
    async fn delete_org(&self, id: &str) -> Result<()>;
}

/// Registry for users.
#[async_trait]
pub trait UserRegistry: Send + Sync {
    /// Create a user.
    async fn create_user(&self, user: &User) -> Result<()>;
    /// Get a user by ID within an organization.
    async fn get_user(&self, org_id: &str, id: &str) -> Result<User>;
    /// Get a user by email within an organization.
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User>;
    /// List users in an organization.
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>>;
    /// Update a user.
    async fn update_user(&self, user: &User) -> Result<()>;
    /// Delete a user by ID.
    async fn delete_user(&self, id: &str) -> Result<()>;
}

/// Registry for teams.
#[async_trait]
pub trait TeamRegistry: Send + Sync {
    /// Create a team.
    async fn create_team(&self, team: &Team) -> Result<()>;
    /// Get a team by ID.
    async fn get_team(&self, id: &str) -> Result<Team>;
    /// List teams in an organization.
    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>>;
    /// Delete a team by ID.
    async fn delete_team(&self, id: &str) -> Result<()>;
}

/// Manager for team members.
#[async_trait]
pub trait TeamMemberManager: Send + Sync {
    /// Add a team member.
    async fn add_team_member(&self, member: &TeamMember) -> Result<()>;
    /// Remove a team member.
    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()>;
    /// List team members.
    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>>;
}

/// Registry for API keys.
#[async_trait]
pub trait ApiKeyRegistry: Send + Sync {
    /// Create an API key.
    async fn create_api_key(&self, key: &ApiKey) -> Result<()>;
    /// Get an API key by ID.
    async fn get_api_key(&self, id: &str) -> Result<ApiKey>;
    /// List API keys in an organization.
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>>;
    /// Revoke an API key.
    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()>;
    /// Delete an API key by ID.
    async fn delete_api_key(&self, id: &str) -> Result<()>;
}

define_aggregate! {
    /// Aggregate trait for org entity management.
    pub trait OrgEntityRepository = OrgRegistry + UserRegistry + TeamRegistry + TeamMemberManager + ApiKeyRegistry;
}

// ============================================================================
// VCS
// ============================================================================

/// Unified repository for VCS entities.
#[async_trait]
pub trait VcsEntityRepository: Send + Sync {
    /// Create a repository.
    async fn create_repository(&self, repo: &Repository) -> Result<()>;
    /// Get a repository by ID.
    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository>;
    /// List repositories for a project.
    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>>;
    /// Update a repository.
    async fn update_repository(&self, repo: &Repository) -> Result<()>;
    /// Delete a repository.
    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()>;

    /// Create a branch.
    async fn create_branch(&self, branch: &Branch) -> Result<()>;
    /// Get a branch by ID.
    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch>;
    /// List branches for a repository.
    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>>;
    /// Update a branch.
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    /// Delete a branch.
    async fn delete_branch(&self, id: &str) -> Result<()>;

    /// Create a worktree.
    async fn create_worktree(&self, wt: &Worktree) -> Result<()>;
    /// Get a worktree by ID.
    async fn get_worktree(&self, id: &str) -> Result<Worktree>;
    /// List worktrees for a repository.
    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>>;
    /// Update a worktree.
    async fn update_worktree(&self, wt: &Worktree) -> Result<()>;
    /// Delete a worktree.
    async fn delete_worktree(&self, id: &str) -> Result<()>;

    /// Create an agent-worktree assignment.
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()>;
    /// Get an assignment by ID.
    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment>;
    /// List assignments for a worktree.
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>>;
    /// Release an assignment.
    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()>;
}

// ============================================================================
// Agent
// ============================================================================

/// Query filters for agent session lookups.
#[derive(Debug, Clone, Default)]
pub struct AgentSessionQuery {
    /// Filter by session summary ID.
    pub session_summary_id: Option<String>,
    /// Filter by parent session ID.
    pub parent_session_id: Option<String>,
    /// Filter by agent type.
    pub agent_type: Option<AgentType>,
    /// Filter by session status.
    pub status: Option<AgentSessionStatus>,
    /// Filter by project ID.
    pub project_id: Option<String>,
    /// Filter by worktree ID.
    pub worktree_id: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

/// Port for agent session persistence.
#[async_trait]
pub trait AgentSessionRepository: Send + Sync {
    /// Create a new agent session.
    async fn create_session(&self, session: &AgentSession) -> Result<()>;
    /// Get an agent session by ID.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    /// Update an existing agent session.
    async fn update_session(&self, session: &AgentSession) -> Result<()>;
    /// List agent sessions matching query filters.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    /// List agent sessions for a project.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;
    /// List agent sessions for a worktree.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
}

/// Port for agent event persistence (delegations, tool calls).
#[async_trait]
pub trait AgentEventRepository: Send + Sync {
    /// Store a delegation record.
    async fn store_delegation(&self, delegation: &Delegation) -> Result<()>;
    /// Store a tool call record.
    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()>;
}

/// Port for agent checkpoint persistence.
#[async_trait]
pub trait AgentCheckpointRepository: Send + Sync {
    /// Store a checkpoint.
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
    /// Get a checkpoint by ID.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Update an existing checkpoint.
    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
}

define_aggregate! {
    /// Aggregate trait for full agent persistence capabilities.
    pub trait AgentRepository = AgentSessionRepository + AgentEventRepository + AgentCheckpointRepository;
}

// ============================================================================
// Issue
// ============================================================================

use crate::entities::project::ProjectIssue;

/// Registry for issues.
#[async_trait]
pub trait IssueRegistry: Send + Sync {
    /// Create an issue.
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Get an issue by ID.
    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue>;
    /// List issues for a project.
    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>>;
    /// Update an issue.
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Delete an issue.
    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()>;
}

/// Registry for issue comments.
#[async_trait]
pub trait IssueCommentRegistry: Send + Sync {
    /// Create a comment.
    async fn create_comment(&self, comment: &IssueComment) -> Result<()>;
    /// Get a comment by ID.
    async fn get_comment(&self, id: &str) -> Result<IssueComment>;
    /// List comments for an issue.
    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>>;
    /// Delete a comment.
    async fn delete_comment(&self, id: &str) -> Result<()>;
}

/// Registry for issue labels.
#[async_trait]
pub trait IssueLabelRegistry: Send + Sync {
    /// Create a label.
    async fn create_label(&self, label: &IssueLabel) -> Result<()>;
    /// Get a label by ID.
    async fn get_label(&self, id: &str) -> Result<IssueLabel>;
    /// List labels for a project.
    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>>;
    /// Delete a label.
    async fn delete_label(&self, id: &str) -> Result<()>;
}

/// Manager for issue label assignments.
#[async_trait]
pub trait IssueLabelAssignmentManager: Send + Sync {
    /// Assign a label to an issue.
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()>;
    /// Unassign a label from an issue.
    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()>;
    /// List labels for an issue.
    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>>;
}

define_aggregate! {
    /// Aggregate trait for issue entity management.
    pub trait IssueEntityRepository = IssueRegistry + IssueCommentRegistry + IssueLabelRegistry + IssueLabelAssignmentManager;
}

// ============================================================================
// Plan
// ============================================================================

/// Registry for plans.
#[async_trait]
pub trait PlanRegistry: Send + Sync {
    /// Create a plan.
    async fn create_plan(&self, plan: &Plan) -> Result<()>;
    /// Get a plan by ID.
    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan>;
    /// List plans for a project.
    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>>;
    /// Update a plan.
    async fn update_plan(&self, plan: &Plan) -> Result<()>;
    /// Delete a plan.
    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()>;
}

/// Registry for plan versions.
#[async_trait]
pub trait PlanVersionRegistry: Send + Sync {
    /// Create a plan version.
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()>;
    /// Get a plan version by ID.
    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion>;
    /// List versions for a plan.
    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>>;
}

/// Registry for plan reviews.
#[async_trait]
pub trait PlanReviewRegistry: Send + Sync {
    /// Create a plan review.
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()>;
    /// Get a plan review by ID.
    async fn get_plan_review(&self, id: &str) -> Result<PlanReview>;
    /// List reviews for a plan version.
    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>>;
}

define_aggregate! {
    /// Aggregate trait for plan entity management.
    pub trait PlanEntityRepository = PlanRegistry + PlanVersionRegistry + PlanReviewRegistry;
}

// ============================================================================
// Memory
// ============================================================================

/// FTS search result with BM25 rank score.
#[derive(Debug, Clone)]
pub struct FtsSearchResult {
    /// Observation ID.
    pub id: String,
    /// BM25 rank score (lower is better, typically negative).
    pub rank: f64,
}

/// Port for observation storage (CRUD, FTS, timeline).
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// Store an observation.
    async fn store_observation(&self, observation: &Observation) -> Result<()>;
    /// Get an observation by ID.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>>;
    /// Find an observation by content hash.
    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>>;
    /// Full-text search returning IDs with BM25 rank scores.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>>;
    /// Delete an observation by ID.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()>;
    /// Get multiple observations by IDs (batch fetch).
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>>;
    /// Get observations in timeline order around an anchor.
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;
    /// Store a session summary.
    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()>;
    /// Get a session summary by session ID.
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>>;
}

// ============================================================================
// Auth
// ============================================================================

/// User information with API key details.
#[derive(Debug, Clone)]
pub struct UserWithApiKey {
    /// The user entity.
    pub user: User,
    /// API key ID.
    pub api_key_id: String,
    /// API key hash.
    pub api_key_hash: String,
}

/// API key information for validation.
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// API key ID.
    pub api_key_id: String,
    /// User ID associated with the key.
    pub user_id: String,
    /// Organization ID (if applicable).
    pub organization_id: Option<String>,
}

/// Port for authentication repository operations.
#[async_trait]
pub trait AuthRepositoryPort: Send + Sync {
    /// Find users by API key hash.
    async fn find_users_by_api_key_hash(&self, key_hash: &str) -> Result<Vec<UserWithApiKey>>;
    /// Verify API key and return key info if valid.
    async fn verify_api_key(&self, key_hash: &str) -> Result<Option<ApiKeyInfo>>;
}

// ============================================================================
// File Hash
// ============================================================================

/// Repository for tracking file content hashes and changes.
#[async_trait]
pub trait FileHashRepository: Send + Sync {
    /// Get hash for a file (returns None if not found or tombstoned).
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>>;
    /// Check if file has changed (returns true if new or hash differs).
    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool>;
    /// Upsert hash for a file (insert or update).
    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()>;
    /// Mark a file as deleted (tombstone).
    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()>;
    /// Get all active file paths for a collection.
    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>>;
    /// Cleanup tombstones older than default TTL.
    async fn cleanup_tombstones(&self) -> Result<u64>;
    /// Cleanup tombstones with custom TTL.
    async fn cleanup_tombstones_with_ttl(&self, ttl: Duration) -> Result<u64>;
    /// Get tombstone count for a collection.
    async fn tombstone_count(&self, collection: &str) -> Result<i64>;
    /// Clear all records for a collection.
    async fn clear_collection(&self, collection: &str) -> Result<u64>;
    /// Compute hash for a local file.
    fn compute_hash(&self, path: &Path) -> Result<String>;
}

// ============================================================================
// Index
// ============================================================================

/// Statistics about a collection's index state.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Number of actively indexed files.
    pub indexed_files: u64,
    /// Timestamp of the most recent indexing operation.
    pub last_indexed_at: Option<i64>,
    /// Whether an indexing operation is currently in progress.
    pub is_indexing: bool,
}

/// Repository for persisting indexing operation state.
#[async_trait]
pub trait IndexRepository: Send + Sync {
    /// Start a new indexing operation for a collection.
    async fn start_indexing(
        &self,
        collection: &CollectionId,
        total_files: usize,
    ) -> Result<OperationId>;
    /// Get the current state of an indexing operation.
    async fn get_operation(&self, operation_id: &OperationId) -> Result<Option<IndexingOperation>>;
    /// Get all indexing operations (active and recent).
    async fn list_operations(&self) -> Result<Vec<IndexingOperation>>;
    /// Get the active operation for a collection, if any.
    async fn get_active_operation(
        &self,
        collection: &CollectionId,
    ) -> Result<Option<IndexingOperation>>;
    /// Update progress of an indexing operation.
    async fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed_files: usize,
    ) -> Result<()>;
    /// Mark an operation as successfully completed.
    async fn complete_operation(&self, operation_id: &OperationId) -> Result<()>;
    /// Mark an operation as failed with an error message.
    async fn fail_operation(&self, operation_id: &OperationId, error: &str) -> Result<()>;
    /// Clear all index data for a collection.
    async fn clear_index(&self, collection: &CollectionId) -> Result<u64>;
    /// Get indexing statistics for a collection.
    async fn get_index_stats(&self, collection: &CollectionId) -> Result<IndexStats>;
}

// ============================================================================
// Project
// ============================================================================

/// Port for project persistence with row-level tenant isolation.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Create a project.
    async fn create(&self, project: &Project) -> Result<()>;
    /// Get a project by ID.
    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project>;
    /// Get a project by name.
    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project>;
    /// Get a project by path.
    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project>;
    /// List projects in an organization.
    async fn list(&self, org_id: &str) -> Result<Vec<Project>>;
    /// Update a project.
    async fn update(&self, project: &Project) -> Result<()>;
    /// Delete a project.
    async fn delete(&self, org_id: &str, id: &str) -> Result<()>;
}

// ============================================================================
// Workflow
// ============================================================================

/// Port for workflow session persistence.
#[async_trait]
pub trait WorkflowSessionRepository: Send + Sync {
    /// Persist a new workflow session.
    async fn create(&self, session: &WorkflowSession) -> Result<()>;
    /// Fetch a workflow session by ID.
    async fn get_by_id(&self, session_id: &str) -> Result<WorkflowSession>;
    /// List workflow sessions for a project.
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<WorkflowSession>>;
    /// Update workflow state with optimistic concurrency.
    async fn update_state(
        &self,
        session_id: &str,
        new_state: WorkflowState,
        version: u32,
    ) -> Result<()>;
}

/// Port for workflow transition audit persistence.
#[async_trait]
pub trait TransitionRepository: Send + Sync {
    /// Record a transition event.
    async fn record(&self, transition: &Transition) -> Result<()>;
    /// List transitions for a workflow session.
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Transition>>;
}
