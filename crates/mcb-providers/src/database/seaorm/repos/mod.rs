//! `SeaORM` repository implementations.
//!
//! This module groups concrete persistence repositories used by the server and
//! infrastructure layers.

#[macro_use]
mod common;
/// Agent repository implementation.
pub mod agent;
/// Entity repository bundle.
mod entity_repository;
/// Indexing repository implementation.
pub mod index;
/// Observation repository implementation.
pub mod observation;
/// Project repository implementation.
pub mod project;
/// Database repository bundle registry integration.
pub mod registry;

/// `SeaORM` agent repository.
pub use agent::SeaOrmAgentRepository;
/// Unified entity repository.
pub use entity_repository::SeaOrmEntityRepository;
/// `SeaORM` indexing repository.
pub use index::SeaOrmIndexRepository;
/// `SeaORM` observation repository.
pub use observation::SeaOrmObservationRepository;
/// `SeaORM` project repository.
pub use project::SeaOrmProjectRepository;

// Sub-modules containing the macro-generated trait implementations.
mod issues;
mod org;
mod plans;
mod teams;
mod vcs;

// Re-export common items for sub-modules using `super::*`
pub(self) use async_trait::async_trait;
pub(self) use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

pub(self) use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
pub(self) use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
pub(self) use mcb_domain::entities::project::ProjectIssue;
pub(self) use mcb_domain::entities::repository::{Branch, Repository};
pub(self) use mcb_domain::entities::team::TeamMember;
pub(self) use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
pub(self) use mcb_domain::entities::{ApiKey, Organization, Team, User};
pub(self) use mcb_domain::error::{Error, Result};
pub(self) use mcb_domain::ports::{
    AgentAssignmentManager, ApiKeyRegistry, IssueCommentRegistry, IssueLabelAssignmentManager,
    IssueLabelRegistry, IssueRegistry, OrgRegistry, PlanRegistry, PlanReviewRegistry,
    PlanVersionRegistry, TeamMemberManager, TeamRegistry, UserRegistry, VcsBranchRegistry,
    VcsRepositoryRegistry, VcsWorktreeRegistry,
};

pub(self) use crate::database::seaorm::entities::{
    agent_worktree_assignment, api_key, branch, issue_comment, issue_label, issue_label_assignment,
    organization, plan, plan_review, plan_version, project_issue, repository, team, team_member,
    user, worktree,
};
pub(self) use common::db_err;
pub(self) use entity_repository::SeaOrmEntityRepository;
