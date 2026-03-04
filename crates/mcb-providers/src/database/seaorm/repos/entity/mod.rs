//! Unified Entity CRUD Repository using `SeaORM`.
//!
//! Implements all entity repository port traits from `mcb-domain` using `SeaORM`
//! for type-safe database access. Covers VCS, Plan, Issue, and Org entity groups.

use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::team::TeamMember;
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::entities::{ApiKey, Organization, Team, User};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    AgentAssignmentManager, ApiKeyRegistry, IssueCommentRegistry, IssueLabelAssignmentManager,
    IssueLabelRegistry, IssueRegistry, OrgRegistry, PlanRegistry, PlanReviewRegistry,
    PlanVersionRegistry, TeamMemberManager, TeamRegistry, UserRegistry, VcsBranchRegistry,
    VcsRepositoryRegistry, VcsWorktreeRegistry,
};

use super::common::db_err;
use crate::database::seaorm::entities::{
    agent_worktree_assignment, api_key, branch, issue_comment, issue_label, issue_label_assignment,
    organization, plan, plan_review, plan_version, project_issue, repository, team, team_member,
    user, worktree,
};

/// Unified SeaORM-backed entity repository implementing all entity CRUD traits.
///
/// This single struct implements `VcsEntityRepository`, `OrgEntityRepository`,
/// `PlanEntityRepository`, and `IssueEntityRepository` — providing a unified
/// persistence layer for all entity types through `SeaORM`.
pub struct SeaOrmEntityRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmEntityRepository {
    /// Creates a new entity repository backed by the given database connection.
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Returns a reference to the underlying database connection.
    #[must_use]
    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }
}

mod issues;
mod org;
mod plans;
mod teams;
mod vcs;
