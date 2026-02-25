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

// ---------------------------------------------------------------------------
// Helper: map SeaORM errors to domain errors
// ---------------------------------------------------------------------------

fn db_err(e: sea_orm::DbErr) -> Error {
    Error::Database {
        message: "Database error".into(),
        source: Some(Box::new(e)),
    }
}

// ==========================================================================
// VCS Entity Repository
// ==========================================================================

#[async_trait]
impl VcsEntityRepository for SeaOrmEntityRepository {
    // -- Repository --

    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        let active: repository::ActiveModel = repo.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        let model = repository::Entity::find_by_id(id)
            .filter(repository::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Repository::from), "Repository", id)
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        let models = repository::Entity::find()
            .filter(repository::Column::OrgId.eq(org_id))
            .filter(repository::Column::ProjectId.eq(project_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Repository::from).collect())
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        let active: repository::ActiveModel = repo.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        let model = repository::Entity::find_by_id(id)
            .filter(repository::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }

    // -- Branch --

    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        let active: branch::ActiveModel = branch.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch> {
        let model = branch::Entity::find_by_id(id)
            .filter(branch::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Branch::from), "Branch", id)
    }

    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>> {
        let models = branch::Entity::find()
            .filter(branch::Column::OrgId.eq(org_id))
            .filter(branch::Column::RepositoryId.eq(repository_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Branch::from).collect())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        let active: branch::ActiveModel = branch.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        let model = branch::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }

    // -- Worktree --

    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        let active: worktree::ActiveModel = wt.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        let model = worktree::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Worktree::from), "Worktree", id)
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        let models = worktree::Entity::find()
            .filter(worktree::Column::RepositoryId.eq(repository_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Worktree::from).collect())
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        let active: worktree::ActiveModel = wt.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        let model = worktree::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }

    // -- Assignment --

    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        let active: agent_worktree_assignment::ActiveModel = asgn.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        let model = agent_worktree_assignment::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(AgentWorktreeAssignment::from), "Assignment", id)
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        let models = agent_worktree_assignment::Entity::find()
            .filter(agent_worktree_assignment::Column::WorktreeId.eq(worktree_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models
            .into_iter()
            .map(AgentWorktreeAssignment::from)
            .collect())
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
        let active: organization::ActiveModel = org.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        let model = organization::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Organization::from), "Organization", id)
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        let models = organization::Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Organization::from).collect())
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        let active: organization::ActiveModel = org.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        let model = organization::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl UserRegistry for SeaOrmEntityRepository {
    async fn create_user(&self, u: &User) -> Result<()> {
        let active: user::ActiveModel = u.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_user(&self, org_id: &str, id: &str) -> Result<User> {
        let model = user::Entity::find_by_id(id)
            .filter(user::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(User::from), "User", id)
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
        let models = user::Entity::find()
            .filter(user::Column::OrgId.eq(org_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(User::from).collect())
    }

    async fn update_user(&self, u: &User) -> Result<()> {
        let active: user::ActiveModel = u.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        let model = user::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl TeamRegistry for SeaOrmEntityRepository {
    async fn create_team(&self, t: &Team) -> Result<()> {
        let active: team::ActiveModel = t.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        let model = team::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Team::from), "Team", id)
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        let models = team::Entity::find()
            .filter(team::Column::OrgId.eq(org_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Team::from).collect())
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        let model = team::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl TeamMemberManager for SeaOrmEntityRepository {
    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        let active: team_member::ActiveModel = member.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        let model = team_member::Entity::find_by_id((team_id.to_owned(), user_id.to_owned()))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        let models = team_member::Entity::find()
            .filter(team_member::Column::TeamId.eq(team_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(TeamMember::from).collect())
    }
}

#[async_trait]
impl ApiKeyRegistry for SeaOrmEntityRepository {
    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        let active: api_key::ActiveModel = key.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        let model = api_key::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(ApiKey::from), "ApiKey", id)
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        let models = api_key::Entity::find()
            .filter(api_key::Column::OrgId.eq(org_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(ApiKey::from).collect())
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
        let model = api_key::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

// ==========================================================================
// Plan Entity Repository
// ==========================================================================

#[async_trait]
impl PlanRegistry for SeaOrmEntityRepository {
    async fn create_plan(&self, p: &Plan) -> Result<()> {
        let active: plan::ActiveModel = p.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        let model = plan::Entity::find_by_id(id)
            .filter(plan::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(Plan::from), "Plan", id)
    }

    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        let models = plan::Entity::find()
            .filter(plan::Column::OrgId.eq(org_id))
            .filter(plan::Column::ProjectId.eq(project_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(Plan::from).collect())
    }

    async fn update_plan(&self, p: &Plan) -> Result<()> {
        let active: plan::ActiveModel = p.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        let model = plan::Entity::find_by_id(id)
            .filter(plan::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl PlanVersionRegistry for SeaOrmEntityRepository {
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        let active: plan_version::ActiveModel = version.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        let model = plan_version::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(PlanVersion::from), "PlanVersion", id)
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        let models = plan_version::Entity::find()
            .filter(plan_version::Column::PlanId.eq(plan_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(PlanVersion::from).collect())
    }
}

#[async_trait]
impl PlanReviewRegistry for SeaOrmEntityRepository {
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        let active: plan_review::ActiveModel = review.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        let model = plan_review::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(PlanReview::from), "PlanReview", id)
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        let models = plan_review::Entity::find()
            .filter(plan_review::Column::PlanVersionId.eq(plan_version_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(PlanReview::from).collect())
    }
}

// ==========================================================================
// Issue Entity Repository
// ==========================================================================

#[async_trait]
impl IssueRegistry for SeaOrmEntityRepository {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        let model = project_issue::Entity::find_by_id(id)
            .filter(project_issue::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(ProjectIssue::from), "Issue", id)
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        let models = project_issue::Entity::find()
            .filter(project_issue::Column::OrgId.eq(org_id))
            .filter(project_issue::Column::ProjectId.eq(project_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(ProjectIssue::from).collect())
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let active: project_issue::ActiveModel = issue.clone().into();
        active.update(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        let model = project_issue::Entity::find_by_id(id)
            .filter(project_issue::Column::OrgId.eq(org_id))
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl IssueCommentRegistry for SeaOrmEntityRepository {
    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        let active: issue_comment::ActiveModel = comment.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        let model = issue_comment::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(IssueComment::from), "IssueComment", id)
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        let models = issue_comment::Entity::find()
            .filter(issue_comment::Column::IssueId.eq(issue_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(IssueComment::from).collect())
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        let model = issue_comment::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl IssueLabelRegistry for SeaOrmEntityRepository {
    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        let active: issue_label::ActiveModel = label.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        let model = issue_label::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Error::not_found_or(model.map(IssueLabel::from), "IssueLabel", id)
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        let models = issue_label::Entity::find()
            .filter(issue_label::Column::OrgId.eq(org_id))
            .filter(issue_label::Column::ProjectId.eq(project_id))
            .all(self.db.as_ref())
            .await
            .map_err(db_err)?;
        Ok(models.into_iter().map(IssueLabel::from).collect())
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        let model = issue_label::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl IssueLabelAssignmentManager for SeaOrmEntityRepository {
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        let active: issue_label_assignment::ActiveModel = assignment.clone().into();
        active.insert(self.db.as_ref()).await.map_err(db_err)?;
        Ok(())
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        let model =
            issue_label_assignment::Entity::find_by_id((issue_id.to_owned(), label_id.to_owned()))
                .one(self.db.as_ref())
                .await
                .map_err(db_err)?;
        if let Some(m) = model {
            m.delete(self.db.as_ref()).await.map_err(db_err)?;
        }
        Ok(())
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
