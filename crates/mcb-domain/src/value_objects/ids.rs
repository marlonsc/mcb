//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#value-objects)
//!
//! Strong-typed UUID identifiers for all domain entities.

define_id!(CollectionId, "Strong typed identifier for a collection");
define_id!(ChunkId, "Strong typed identifier for a code chunk");
define_id!(RepositoryId, "Strong typed identifier for a VCS repository");
define_id!(
    SessionId,
    "Strong typed identifier for an agent or workflow session"
);
define_id!(
    ObservationId,
    "Strong typed identifier for a memory observation"
);
define_id!(
    OperationId,
    "Strong typed identifier for an indexing operation"
);
define_id!(
    OrgId,
    "Strong typed identifier for an organization (tenant isolation)"
);
define_id!(UserId, "Strong typed identifier for a user");
define_id!(TeamId, "Strong typed identifier for a team");
define_id!(ApiKeyId, "Strong typed identifier for an API key");
define_id!(BranchId, "Strong typed identifier for a tracked branch");
define_id!(WorktreeId, "Strong typed identifier for a worktree");
define_id!(
    AssignmentId,
    "Strong typed identifier for an agent-worktree assignment"
);
define_id!(PlanId, "Strong typed identifier for a plan");
define_id!(PlanVersionId, "Strong typed identifier for a plan version");
define_id!(PlanReviewId, "Strong typed identifier for a plan review");
define_id!(IssueId, "Strong typed identifier for a project issue");
define_id!(
    IssueCommentId,
    "Strong typed identifier for an issue comment"
);
define_id!(IssueLabelId, "Strong typed identifier for an issue label");

define_id!(CodebaseId, "Strong typed identifier for a codebase");
define_id!(FileId, "Strong typed identifier for a file");
define_id!(FunctionId, "Strong typed identifier for a function");
define_id!(ClassId, "Strong typed identifier for a class");

define_id!(TeamMemberId, "Strong typed identifier for a team member");
define_id!(
    IssueLabelAssignmentId,
    "Strong typed identifier for an issue label assignment"
);

define_id!(ProjectId, "Strong typed identifier for a project");
define_id!(PhaseId, "Strong typed identifier for a project phase");
define_id!(
    DependencyId,
    "Strong typed identifier for a project dependency"
);
define_id!(DecisionId, "Strong typed identifier for a project decision");
define_id!(
    TransitionId,
    "Strong typed identifier for a workflow transition"
);
define_id!(SubmoduleId, "Strong typed identifier for a submodule");
define_id!(
    DelegationId,
    "Strong typed identifier for an agent delegation"
);
define_id!(ToolCallId, "Strong typed identifier for a tool call");
define_id!(
    CheckpointId,
    "Strong typed identifier for an agent checkpoint"
);
define_id!(
    ErrorPatternId,
    "Strong typed identifier for an error pattern"
);
define_id!(
    ErrorPatternMatchId,
    "Strong typed identifier for an error pattern match"
);
define_id!(
    ExecutionId,
    "Strong typed identifier for an execution record"
);
define_id!(
    QualityGateId,
    "Strong typed identifier for a quality gate result"
);
define_id!(DiffId, "Strong typed identifier for a file or ref diff");
define_id!(
    SnapshotId,
    "Strong typed identifier for a codebase snapshot"
);
define_id!(
    SessionSummaryId,
    "Strong typed identifier for a session summary"
);
define_id!(
    OriginContextId,
    "Strong typed identifier for an origin context"
);
define_id!(
    MemorySearchId,
    "Strong typed identifier for a memory search entry"
);
