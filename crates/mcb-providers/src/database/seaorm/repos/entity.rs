//! Unified Entity CRUD Repository using `SeaORM`.
//!
//! Implements all entity repository port traits from `mcb-domain` using `SeaORM`
//! for type-safe database access. Covers VCS, Plan, Issue, and Org entity groups.

use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
};

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::team::TeamMember;
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::entities::{ApiKey, Organization, Team, User};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    ApiKeyRegistry, IssueCommentRegistry, IssueLabelAssignmentManager, IssueLabelRegistry,
    IssueRegistry, OrgRegistry, PlanRegistry, PlanReviewRegistry, PlanVersionRegistry,
    TeamMemberManager, TeamRegistry, UserRegistry, VcsEntityRepository,
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

macro_rules! sea_insert {
    ($self:expr, $mod:ident, $item:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        active.insert($self.db()).await.map_err(db_err)?;
        Ok(())
    }};
}

macro_rules! sea_get {
    ($self:expr, $mod:ident, $type:ty, $label:literal, $id:expr) => {{
        let model = $mod::Entity::find_by_id($id)
            .one($self.db())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

macro_rules! sea_get_filtered {
    ($self:expr, $mod:ident, $type:ty, $label:literal, $id:expr, $($col:expr => $val:expr),+) => {{
        let model = $mod::Entity::find_by_id($id)
            $(.filter($col.eq($val)))+
            .one($self.db())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

macro_rules! sea_list {
    ($self:expr, $mod:ident, $type:ty $(, $col:expr => $val:expr)*) => {{
        let models = $mod::Entity::find()
            $(.filter($col.eq($val)))*
            .all($self.db())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(<$type>::from).collect())
    }};
}

macro_rules! sea_update {
    ($self:expr, $mod:ident, $item:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        active.update($self.db()).await.map_err(db_err)?;
        Ok(())
    }};
}

macro_rules! sea_delete {
    ($self:expr, $mod:ident, $id:expr) => {{
        if let Some(m) = $mod::Entity::find_by_id($id)
            .one($self.db())
            .await
            .map_err(db_err)?
        {
            m.delete($self.db()).await.map_err(db_err)?;
        }
        Ok(())
    }};
}

macro_rules! sea_delete_filtered {
    ($self:expr, $mod:ident, $id:expr, $($col:expr => $val:expr),+) => {{
        if let Some(m) = $mod::Entity::find_by_id($id)
            $(.filter($col.eq($val)))+
            .one($self.db())
            .await
            .map_err(db_err)?
        {
            m.delete($self.db()).await.map_err(db_err)?;
        }
        Ok(())
    }};
}

// ==========================================================================
// VCS Entity Repository
// ==========================================================================

#[async_trait]
impl VcsEntityRepository for SeaOrmEntityRepository {
    // -- Repository --

    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        sea_insert!(self, repository, repo)
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        sea_get_filtered!(self, repository, Repository, "Repository", id, repository::Column::OrgId => org_id)
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        sea_list!(self, repository, Repository, repository::Column::OrgId => org_id, repository::Column::ProjectId => project_id)
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        sea_update!(self, repository, repo)
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, repository, id, repository::Column::OrgId => org_id)
    }

    // -- Branch --

    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        sea_insert!(self, branch, branch)
    }

    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch> {
        sea_get_filtered!(self, branch, Branch, "Branch", id, branch::Column::OrgId => org_id)
    }

    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>> {
        sea_list!(self, branch, Branch, branch::Column::OrgId => org_id, branch::Column::RepositoryId => repository_id)
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        sea_update!(self, branch, branch)
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        sea_delete!(self, branch, id)
    }

    // -- Worktree --

    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_insert!(self, worktree, wt)
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        sea_get!(self, worktree, Worktree, "Worktree", id)
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        sea_list!(self, worktree, Worktree, worktree::Column::RepositoryId => repository_id)
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_update!(self, worktree, wt)
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        sea_delete!(self, worktree, id)
    }

    // -- Assignment --

    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        sea_insert!(self, agent_worktree_assignment, asgn)
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        sea_get!(
            self,
            agent_worktree_assignment,
            AgentWorktreeAssignment,
            "Assignment",
            id
        )
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        sea_list!(self, agent_worktree_assignment, AgentWorktreeAssignment, agent_worktree_assignment::Column::WorktreeId => worktree_id)
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = agent_worktree_assignment::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        let m = Error::not_found_or(model, "Assignment", id)?;

        let mut active: agent_worktree_assignment::ActiveModel = m.into();
        active.released_at = ActiveValue::Set(Some(released_at));
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }
}

// ==========================================================================
// Organization Entity Repository
// ==========================================================================

#[async_trait]
impl OrgRegistry for SeaOrmEntityRepository {
    async fn create_org(&self, org: &Organization) -> Result<()> {
        sea_insert!(self, organization, org)
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        sea_get!(self, organization, Organization, "Organization", id)
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        sea_list!(self, organization, Organization)
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        sea_update!(self, organization, org)
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        sea_delete!(self, organization, id)
    }
}

#[async_trait]
impl UserRegistry for SeaOrmEntityRepository {
    async fn create_user(&self, u: &User) -> Result<()> {
        sea_insert!(self, user, u)
    }

    async fn get_user(&self, org_id: &str, id: &str) -> Result<User> {
        sea_get_filtered!(self, user, User, "User", id, user::Column::OrgId => org_id)
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        let model = user::Entity::find()
            .filter(user::Column::OrgId.eq(org_id))
            .filter(user::Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(User::from), "User", email)
    }

    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        sea_list!(self, user, User, user::Column::OrgId => org_id)
    }

    async fn update_user(&self, u: &User) -> Result<()> {
        sea_update!(self, user, u)
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        sea_delete!(self, user, id)
    }
}

#[async_trait]
impl TeamRegistry for SeaOrmEntityRepository {
    async fn create_team(&self, t: &Team) -> Result<()> {
        sea_insert!(self, team, t)
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        sea_get!(self, team, Team, "Team", id)
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        sea_list!(self, team, Team, team::Column::OrgId => org_id)
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        sea_delete!(self, team, id)
    }
}

#[async_trait]
impl TeamMemberManager for SeaOrmEntityRepository {
    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        sea_insert!(self, team_member, member)
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        sea_delete!(self, team_member, (team_id.to_owned(), user_id.to_owned()))
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        sea_list!(self, team_member, TeamMember, team_member::Column::TeamId => team_id)
    }
}

#[async_trait]
impl ApiKeyRegistry for SeaOrmEntityRepository {
    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        sea_insert!(self, api_key, key)
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        sea_get!(self, api_key, ApiKey, "ApiKey", id)
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        sea_list!(self, api_key, ApiKey, api_key::Column::OrgId => org_id)
    }

    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = api_key::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        let m = Error::not_found_or(model, "ApiKey", id)?;

        let mut active: api_key::ActiveModel = m.into();
        active.revoked_at = ActiveValue::Set(Some(revoked_at));
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        sea_delete!(self, api_key, id)
    }
}

// ==========================================================================
// Plan Entity Repository
// ==========================================================================

#[async_trait]
impl PlanRegistry for SeaOrmEntityRepository {
    async fn create_plan(&self, p: &Plan) -> Result<()> {
        sea_insert!(self, plan, p)
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        sea_get_filtered!(self, plan, Plan, "Plan", id, plan::Column::OrgId => org_id)
    }

    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        sea_list!(self, plan, Plan, plan::Column::OrgId => org_id, plan::Column::ProjectId => project_id)
    }

    async fn update_plan(&self, p: &Plan) -> Result<()> {
        sea_update!(self, plan, p)
    }

    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, plan, id, plan::Column::OrgId => org_id)
    }
}

#[async_trait]
impl PlanVersionRegistry for SeaOrmEntityRepository {
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        sea_insert!(self, plan_version, version)
    }

    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        sea_get!(self, plan_version, PlanVersion, "PlanVersion", id)
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        sea_list!(self, plan_version, PlanVersion, plan_version::Column::PlanId => plan_id)
    }
}

#[async_trait]
impl PlanReviewRegistry for SeaOrmEntityRepository {
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        sea_insert!(self, plan_review, review)
    }

    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        sea_get!(self, plan_review, PlanReview, "PlanReview", id)
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        sea_list!(self, plan_review, PlanReview, plan_review::Column::PlanVersionId => plan_version_id)
    }
}

// ==========================================================================
// Issue Entity Repository
// ==========================================================================

#[async_trait]
impl IssueRegistry for SeaOrmEntityRepository {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_insert!(self, project_issue, issue)
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        sea_get_filtered!(self, project_issue, ProjectIssue, "Issue", id, project_issue::Column::OrgId => org_id)
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        sea_list!(self, project_issue, ProjectIssue, project_issue::Column::OrgId => org_id, project_issue::Column::ProjectId => project_id)
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_update!(self, project_issue, issue)
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, project_issue, id, project_issue::Column::OrgId => org_id)
    }
}

#[async_trait]
impl IssueCommentRegistry for SeaOrmEntityRepository {
    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        sea_insert!(self, issue_comment, comment)
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        sea_get!(self, issue_comment, IssueComment, "IssueComment", id)
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        sea_list!(self, issue_comment, IssueComment, issue_comment::Column::IssueId => issue_id)
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        sea_delete!(self, issue_comment, id)
    }
}

#[async_trait]
impl IssueLabelRegistry for SeaOrmEntityRepository {
    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        sea_insert!(self, issue_label, label)
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        sea_get!(self, issue_label, IssueLabel, "IssueLabel", id)
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        sea_list!(self, issue_label, IssueLabel, issue_label::Column::OrgId => org_id, issue_label::Column::ProjectId => project_id)
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        sea_delete!(self, issue_label, id)
    }
}

#[async_trait]
impl IssueLabelAssignmentManager for SeaOrmEntityRepository {
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        sea_insert!(self, issue_label_assignment, assignment)
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        sea_delete!(
            self,
            issue_label_assignment,
            (issue_id.to_owned(), label_id.to_owned())
        )
    }

    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>> {
        let assignments = issue_label_assignment::Entity::find()
            .filter(issue_label_assignment::Column::IssueId.eq(issue_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;

        if assignments.is_empty() {
            return Ok(vec![]);
        }

        let label_ids: Vec<String> = assignments.into_iter().map(|a| a.label_id).collect();
        let labels = issue_label::Entity::find()
            .filter(issue_label::Column::Id.is_in(label_ids))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;

        Ok(labels.into_iter().map(IssueLabel::from).collect())
    }
}
