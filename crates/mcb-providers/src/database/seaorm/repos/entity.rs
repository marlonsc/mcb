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

#[cfg(test)]
mod tests {
    use super::*;
    use mcb_domain::entities::plan::{PlanStatus, ReviewVerdict};
    use mcb_domain::entities::project::{IssueStatus, IssueType};
    use mcb_domain::entities::repository::VcsType;
    use mcb_domain::entities::team::TeamMemberRole;
    use mcb_domain::entities::user::UserRole;
    use mcb_domain::entities::worktree::WorktreeStatus;
    use mcb_domain::value_objects::ids::{IssueLabelAssignmentId, TeamMemberId};
    use sea_orm_migration::MigratorTrait;

    use crate::database::seaorm::migration::Migrator;

    async fn setup_db() -> Arc<DatabaseConnection> {
        let db = sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory SQLite");
        Migrator::up(&db, None).await.expect("migration up");
        Arc::new(db)
    }

    // -- Org seed helper (many entities need an org to exist first) --

    async fn seed_org(repo: &SeaOrmEntityRepository) {
        let org = Organization {
            id: "org-001".into(),
            name: "Test Org".into(),
            slug: "test-org".into(),
            settings_json: "{}".into(),
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_org(&org).await.expect("seed org");
    }

    async fn seed_user(repo: &SeaOrmEntityRepository) {
        seed_org(repo).await;
        let user = User {
            id: "usr-001".into(),
            org_id: "org-001".into(),
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            role: UserRole::Admin,
            api_key_hash: None,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_user(&user).await.expect("seed user");
    }

    // ======================================================================
    // VCS: Repository CRUD
    // ======================================================================

    #[tokio::test]
    async fn repository_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        // Seed a project (required FK)
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test Project".into()),
            path: ActiveValue::Set("/tmp/proj".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let r = Repository {
            id: "repo-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "mcb".into(),
            url: "https://github.com/user/mcb.git".into(),
            local_path: "/home/user/mcb".into(),
            vcs_type: VcsType::Git,
            created_at: 1700000000,
            updated_at: 1700000001,
        };

        // Create
        repo.create_repository(&r).await.expect("create");

        // Get
        let got = repo
            .get_repository("org-001", "repo-001")
            .await
            .expect("get");
        assert_eq!(got.name, "mcb");
        assert_eq!(got.vcs_type, VcsType::Git);

        // List
        let list = repo
            .list_repositories("org-001", "proj-001")
            .await
            .expect("list");
        assert_eq!(list.len(), 1);

        // Update
        let mut updated = r.clone();
        updated.name = "mcb-updated".into();
        repo.update_repository(&updated).await.expect("update");
        let got2 = repo
            .get_repository("org-001", "repo-001")
            .await
            .expect("get after update");
        assert_eq!(got2.name, "mcb-updated");

        // Delete
        repo.delete_repository("org-001", "repo-001")
            .await
            .expect("delete");
        let list2 = repo
            .list_repositories("org-001", "proj-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // VCS: Branch CRUD
    // ======================================================================

    #[tokio::test]
    async fn branch_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        // Seed project + repository
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let r = Repository {
            id: "repo-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "mcb".into(),
            url: "https://github.com/user/mcb.git".into(),
            local_path: "/home/user/mcb".into(),
            vcs_type: VcsType::Git,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_repository(&r).await.expect("seed repo");

        let b = Branch {
            id: "br-001".into(),
            org_id: "org-001".into(),
            repository_id: "repo-001".into(),
            name: "main".into(),
            is_default: true,
            head_commit: "abc123".into(),
            upstream: Some("origin/main".into()),
            created_at: 1700000000,
        };

        repo.create_branch(&b).await.expect("create");
        let got = repo.get_branch("org-001", "br-001").await.expect("get");
        assert_eq!(got.name, "main");
        assert!(got.is_default);

        let list = repo.list_branches("org-001", "repo-001").await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = b.clone();
        updated.head_commit = "def456".into();
        repo.update_branch(&updated).await.expect("update");
        let got2 = repo.get_branch("org-001", "br-001").await.expect("get after update");
        assert_eq!(got2.head_commit, "def456");

        repo.delete_branch("br-001").await.expect("delete");
        let list2 = repo
            .list_branches("org-001", "repo-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // VCS: Worktree CRUD
    // ======================================================================

    #[tokio::test]
    async fn worktree_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        // Seed project + repository + branch
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let r = Repository {
            id: "repo-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "mcb".into(),
            url: "https://github.com/user/mcb.git".into(),
            local_path: "/home/user/mcb".into(),
            vcs_type: VcsType::Git,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_repository(&r).await.expect("seed repo");

        let b = Branch {
            id: "br-001".into(),
            org_id: "org-001".into(),
            repository_id: "repo-001".into(),
            name: "main".into(),
            is_default: true,
            head_commit: "abc123".into(),
            upstream: None,
            created_at: 1700000000,
        };
        repo.create_branch(&b).await.expect("seed branch");

        let wt = Worktree {
            id: "wt-001".into(),
            repository_id: "repo-001".into(),
            branch_id: "br-001".into(),
            path: "/tmp/worktree".into(),
            status: WorktreeStatus::Active,
            assigned_agent_id: None,
            created_at: 1700000000,
            updated_at: 1700000000,
        };

        repo.create_worktree(&wt).await.expect("create");
        let got = repo.get_worktree("wt-001").await.expect("get");
        assert_eq!(got.path, "/tmp/worktree");

        let list = repo.list_worktrees("repo-001").await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = wt.clone();
        updated.status = WorktreeStatus::InUse;
        repo.update_worktree(&updated).await.expect("update");
        let got2 = repo.get_worktree("wt-001").await.expect("get after update");
        assert_eq!(got2.status, WorktreeStatus::InUse);

        repo.delete_worktree("wt-001").await.expect("delete");
        let list2 = repo
            .list_worktrees("repo-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // VCS: Assignment CRUD
    // ======================================================================

    #[tokio::test]
    async fn assignment_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        // Seed project + repository + branch + worktree + agent_session
        use crate::database::seaorm::entities::{agent_session, project};
        use sea_orm::ActiveValue;

        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let r = Repository {
            id: "repo-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "mcb".into(),
            url: "https://github.com/user/mcb.git".into(),
            local_path: "/home/user/mcb".into(),
            vcs_type: VcsType::Git,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_repository(&r).await.expect("seed repo");

        let b = Branch {
            id: "br-001".into(),
            org_id: "org-001".into(),
            repository_id: "repo-001".into(),
            name: "main".into(),
            is_default: true,
            head_commit: "abc123".into(),
            upstream: None,
            created_at: 1700000000,
        };
        repo.create_branch(&b).await.expect("seed branch");

        let wt = Worktree {
            id: "wt-001".into(),
            repository_id: "repo-001".into(),
            branch_id: "br-001".into(),
            path: "/tmp/worktree".into(),
            status: WorktreeStatus::Active,
            assigned_agent_id: None,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_worktree(&wt).await.expect("seed worktree");

        // Seed agent session
        let ses = agent_session::ActiveModel {
            id: ActiveValue::Set("ses-001".into()),
            project_id: ActiveValue::Set(Some("proj-001".into())),
            worktree_id: ActiveValue::Set(Some("wt-001".into())),
            session_summary_id: ActiveValue::Set(String::new()),
            parent_session_id: ActiveValue::Set(None),
            agent_type: ActiveValue::Set("build".into()),
            model: ActiveValue::Set("claude".into()),
            status: ActiveValue::Set("active".into()),
            prompt_summary: ActiveValue::Set(None),
            result_summary: ActiveValue::Set(None),
            started_at: ActiveValue::Set(1700000000),
            ended_at: ActiveValue::Set(None),
            duration_ms: ActiveValue::Set(None),
            token_count: ActiveValue::Set(None),
            tool_calls_count: ActiveValue::Set(None),
            delegations_count: ActiveValue::Set(None),
        };
        ses.insert(repo.db.as_ref())
            .await
            .expect("seed agent session");

        let asgn = AgentWorktreeAssignment {
            id: "asgn-001".into(),
            agent_session_id: "ses-001".into(),
            worktree_id: "wt-001".into(),
            assigned_at: 1700000000,
            released_at: None,
        };

        repo.create_assignment(&asgn).await.expect("create");
        let got = repo.get_assignment("asgn-001").await.expect("get");
        assert_eq!(got.agent_session_id, "ses-001");
        assert!(got.released_at.is_none());

        let list = repo
            .list_assignments_by_worktree("wt-001")
            .await
            .expect("list");
        assert_eq!(list.len(), 1);

        repo.release_assignment("asgn-001", 1700001000)
            .await
            .expect("release");
        let got2 = repo
            .get_assignment("asgn-001")
            .await
            .expect("get after release");
        assert_eq!(got2.released_at, Some(1700001000));
    }

    // ======================================================================
    // Org: Organization CRUD
    // ======================================================================

    #[tokio::test]
    async fn organization_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);

        let org = Organization {
            id: "org-001".into(),
            name: "Acme Corp".into(),
            slug: "acme-corp".into(),
            settings_json: r#"{"theme":"dark"}"#.into(),
            created_at: 1700000000,
            updated_at: 1700000001,
        };

        repo.create_org(&org).await.expect("create");
        let got = repo.get_org("org-001").await.expect("get");
        assert_eq!(got.name, "Acme Corp");

        let list = repo.list_orgs().await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = org.clone();
        updated.name = "Acme Updated".into();
        repo.update_org(&updated).await.expect("update");
        let got2 = repo.get_org("org-001").await.expect("get after update");
        assert_eq!(got2.name, "Acme Updated");

        repo.delete_org("org-001").await.expect("delete");
        let list2 = repo.list_orgs().await.expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Org: User CRUD
    // ======================================================================

    #[tokio::test]
    async fn user_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        let u = User {
            id: "usr-001".into(),
            org_id: "org-001".into(),
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            role: UserRole::Admin,
            api_key_hash: Some("hash123".into()),
            created_at: 1700000000,
            updated_at: 1700000001,
        };

        repo.create_user(&u).await.expect("create");
        let got = repo.get_user("org-001", "usr-001").await.expect("get");
        assert_eq!(got.email, "alice@example.com");

        let got_email = repo
            .get_user_by_email("org-001", "alice@example.com")
            .await
            .expect("get by email");
        assert_eq!(got_email.id, "usr-001");

        let list = repo.list_users("org-001").await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = u.clone();
        updated.display_name = "Alice Updated".into();
        repo.update_user(&updated).await.expect("update");
        let got2 = repo.get_user("org-001", "usr-001").await.expect("get after update");
        assert_eq!(got2.display_name, "Alice Updated");

        repo.delete_user("usr-001").await.expect("delete");
        let list2 = repo.list_users("org-001").await.expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Org: Team CRUD
    // ======================================================================

    #[tokio::test]
    async fn team_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        let t = Team {
            id: "team-001".into(),
            org_id: "org-001".into(),
            name: "Backend Team".into(),
            created_at: 1700000000,
        };

        repo.create_team(&t).await.expect("create");
        let got = repo.get_team("team-001").await.expect("get");
        assert_eq!(got.name, "Backend Team");

        let list = repo.list_teams("org-001").await.expect("list");
        assert_eq!(list.len(), 1);

        repo.delete_team("team-001").await.expect("delete");
        let list2 = repo.list_teams("org-001").await.expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Org: TeamMember CRUD
    // ======================================================================

    #[tokio::test]
    async fn team_member_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        let t = Team {
            id: "team-001".into(),
            org_id: "org-001".into(),
            name: "Backend Team".into(),
            created_at: 1700000000,
        };
        repo.create_team(&t).await.expect("seed team");

        let member = TeamMember {
            id: TeamMemberId::from("team-001:usr-001"),
            team_id: "team-001".into(),
            user_id: "usr-001".into(),
            role: TeamMemberRole::Lead,
            joined_at: 1700000000,
        };

        repo.add_team_member(&member).await.expect("add");
        let list = repo.list_team_members("team-001").await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].role, TeamMemberRole::Lead);

        repo.remove_team_member("team-001", "usr-001")
            .await
            .expect("remove");
        let list2 = repo
            .list_team_members("team-001")
            .await
            .expect("list after remove");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Org: ApiKey CRUD
    // ======================================================================

    #[tokio::test]
    async fn api_key_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        let key = ApiKey {
            id: "key-001".into(),
            org_id: "org-001".into(),
            user_id: "usr-001".into(),
            key_hash: "sha256:abc123".into(),
            name: "CI Key".into(),
            scopes_json: r#"["read","write"]"#.into(),
            expires_at: Some(1800000000),
            created_at: 1700000000,
            revoked_at: None,
        };

        repo.create_api_key(&key).await.expect("create");
        let got = repo.get_api_key("key-001").await.expect("get");
        assert_eq!(got.name, "CI Key");
        assert!(got.revoked_at.is_none());

        let list = repo.list_api_keys("org-001").await.expect("list");
        assert_eq!(list.len(), 1);

        repo.revoke_api_key("key-001", 1700050000)
            .await
            .expect("revoke");
        let got2 = repo.get_api_key("key-001").await.expect("get after revoke");
        assert_eq!(got2.revoked_at, Some(1700050000));

        repo.delete_api_key("key-001").await.expect("delete");
        let list2 = repo
            .list_api_keys("org-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Plan: Plan CRUD
    // ======================================================================

    #[tokio::test]
    async fn plan_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let p = Plan {
            id: "plan-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            title: "v0.3.0 Roadmap".into(),
            description: "SeaORM migration plan".into(),
            status: PlanStatus::Active,
            created_by: "usr-001".into(),
            created_at: 1700000000,
            updated_at: 1700000001,
        };

        repo.create_plan(&p).await.expect("create");
        let got = repo.get_plan("org-001", "plan-001").await.expect("get");
        assert_eq!(got.title, "v0.3.0 Roadmap");
        assert_eq!(got.status, PlanStatus::Active);

        let list = repo.list_plans("org-001", "proj-001").await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = p.clone();
        updated.status = PlanStatus::Completed;
        repo.update_plan(&updated).await.expect("update");
        let got2 = repo
            .get_plan("org-001", "plan-001")
            .await
            .expect("get after update");
        assert_eq!(got2.status, PlanStatus::Completed);

        repo.delete_plan("org-001", "plan-001")
            .await
            .expect("delete");
        let list2 = repo
            .list_plans("org-001", "proj-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Plan: PlanVersion CRUD
    // ======================================================================

    #[tokio::test]
    async fn plan_version_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project + plan
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let p = Plan {
            id: "plan-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            title: "v0.3.0".into(),
            description: "Plan".into(),
            status: PlanStatus::Draft,
            created_by: "usr-001".into(),
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_plan(&p).await.expect("seed plan");

        let v = PlanVersion {
            id: "pv-001".into(),
            org_id: "org-001".into(),
            plan_id: "plan-001".into(),
            version_number: 1,
            content_json: r#"{"tasks":[]}"#.into(),
            change_summary: "Initial version".into(),
            created_by: "usr-001".into(),
            created_at: 1700000000,
        };

        repo.create_plan_version(&v).await.expect("create");
        let got = repo.get_plan_version("pv-001").await.expect("get");
        assert_eq!(got.version_number, 1);

        let list = repo
            .list_plan_versions_by_plan("plan-001")
            .await
            .expect("list");
        assert_eq!(list.len(), 1);
    }

    // ======================================================================
    // Plan: PlanReview CRUD
    // ======================================================================

    #[tokio::test]
    async fn plan_review_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project + plan + version + reviewer
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let p = Plan {
            id: "plan-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            title: "v0.3.0".into(),
            description: "Plan".into(),
            status: PlanStatus::Draft,
            created_by: "usr-001".into(),
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_plan(&p).await.expect("seed plan");

        let v = PlanVersion {
            id: "pv-001".into(),
            org_id: "org-001".into(),
            plan_id: "plan-001".into(),
            version_number: 1,
            content_json: "{}".into(),
            change_summary: "Init".into(),
            created_by: "usr-001".into(),
            created_at: 1700000000,
        };
        repo.create_plan_version(&v).await.expect("seed version");

        // Seed reviewer user
        let reviewer = User {
            id: "usr-002".into(),
            org_id: "org-001".into(),
            email: "bob@example.com".into(),
            display_name: "Bob".into(),
            role: UserRole::Member,
            api_key_hash: None,
            created_at: 1700000000,
            updated_at: 1700000000,
        };
        repo.create_user(&reviewer).await.expect("seed reviewer");

        let review = PlanReview {
            id: "pr-001".into(),
            org_id: "org-001".into(),
            plan_version_id: "pv-001".into(),
            reviewer_id: "usr-002".into(),
            verdict: ReviewVerdict::Approved,
            feedback: "Looks good!".into(),
            created_at: 1700000000,
        };

        repo.create_plan_review(&review).await.expect("create");
        let got = repo.get_plan_review("pr-001").await.expect("get");
        assert_eq!(got.verdict, ReviewVerdict::Approved);

        let list = repo
            .list_plan_reviews_by_version("pv-001")
            .await
            .expect("list");
        assert_eq!(list.len(), 1);
    }

    // ======================================================================
    // Issue: ProjectIssue CRUD
    // ======================================================================

    #[tokio::test]
    async fn issue_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let issue = ProjectIssue {
            id: "iss-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            created_by: "usr-001".into(),
            phase_id: None,
            title: "Fix auth bug".into(),
            description: "Auth fails on refresh".into(),
            issue_type: IssueType::Bug,
            status: IssueStatus::Open,
            priority: 1,
            assignee: Some("usr-001".into()),
            labels: vec!["bug".into()],
            estimated_minutes: Some(120),
            actual_minutes: None,
            notes: String::new(),
            design: String::new(),
            parent_issue_id: None,
            created_at: 1700000000,
            updated_at: 1700000001,
            closed_at: None,
            closed_reason: String::new(),
        };

        repo.create_issue(&issue).await.expect("create");
        let got = repo.get_issue("org-001", "iss-001").await.expect("get");
        assert_eq!(got.title, "Fix auth bug");
        assert_eq!(got.issue_type, IssueType::Bug);

        let list = repo.list_issues("org-001", "proj-001").await.expect("list");
        assert_eq!(list.len(), 1);

        let mut updated = issue.clone();
        updated.status = IssueStatus::Resolved;
        repo.update_issue(&updated).await.expect("update");
        let got2 = repo
            .get_issue("org-001", "iss-001")
            .await
            .expect("get after update");
        assert_eq!(got2.status, IssueStatus::Resolved);

        repo.delete_issue("org-001", "iss-001")
            .await
            .expect("delete");
        let list2 = repo
            .list_issues("org-001", "proj-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Issue: IssueComment CRUD
    // ======================================================================

    #[tokio::test]
    async fn issue_comment_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project + issue
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let issue = ProjectIssue {
            id: "iss-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            created_by: "usr-001".into(),
            phase_id: None,
            title: "Bug".into(),
            description: "Desc".into(),
            issue_type: IssueType::Bug,
            status: IssueStatus::Open,
            priority: 2,
            assignee: None,
            labels: vec![],
            estimated_minutes: None,
            actual_minutes: None,
            notes: String::new(),
            design: String::new(),
            parent_issue_id: None,
            created_at: 1700000000,
            updated_at: 1700000000,
            closed_at: None,
            closed_reason: String::new(),
        };
        repo.create_issue(&issue).await.expect("seed issue");

        let comment = IssueComment {
            id: "cmt-001".into(),
            issue_id: "iss-001".into(),
            author_id: "usr-001".into(),
            content: "This looks like a race condition".into(),
            created_at: 1700000000,
        };

        repo.create_comment(&comment).await.expect("create");
        let got = repo.get_comment("cmt-001").await.expect("get");
        assert_eq!(got.content, "This looks like a race condition");

        let list = repo.list_comments_by_issue("iss-001").await.expect("list");
        assert_eq!(list.len(), 1);

        repo.delete_comment("cmt-001").await.expect("delete");
        let list2 = repo
            .list_comments_by_issue("iss-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Issue: IssueLabel CRUD
    // ======================================================================

    #[tokio::test]
    async fn issue_label_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_org(&repo).await;

        // Seed project
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let label = IssueLabel {
            id: "lbl-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "bug".into(),
            color: "#ff0000".into(),
            created_at: 1700000000,
        };

        repo.create_label(&label).await.expect("create");
        let got = repo.get_label("lbl-001").await.expect("get");
        assert_eq!(got.name, "bug");
        assert_eq!(got.color, "#ff0000");

        let list = repo.list_labels("org-001", "proj-001").await.expect("list");
        assert_eq!(list.len(), 1);

        repo.delete_label("lbl-001").await.expect("delete");
        let list2 = repo
            .list_labels("org-001", "proj-001")
            .await
            .expect("list after delete");
        assert!(list2.is_empty());
    }

    // ======================================================================
    // Issue: IssueLabelAssignment CRUD
    // ======================================================================

    #[tokio::test]
    async fn issue_label_assignment_crud_cycle() {
        let db = setup_db().await;
        let repo = SeaOrmEntityRepository::new(db);
        seed_user(&repo).await;

        // Seed project + issue + label
        use crate::database::seaorm::entities::project;
        use sea_orm::ActiveValue;
        let proj = project::ActiveModel {
            id: ActiveValue::Set("proj-001".into()),
            org_id: ActiveValue::Set("org-001".into()),
            name: ActiveValue::Set("Test".into()),
            path: ActiveValue::Set("/tmp".into()),
            created_at: ActiveValue::Set(1700000000),
            updated_at: ActiveValue::Set(1700000000),
        };
        proj.insert(repo.db.as_ref()).await.expect("seed project");

        let issue = ProjectIssue {
            id: "iss-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            created_by: "usr-001".into(),
            phase_id: None,
            title: "Bug".into(),
            description: "Desc".into(),
            issue_type: IssueType::Bug,
            status: IssueStatus::Open,
            priority: 2,
            assignee: None,
            labels: vec![],
            estimated_minutes: None,
            actual_minutes: None,
            notes: String::new(),
            design: String::new(),
            parent_issue_id: None,
            created_at: 1700000000,
            updated_at: 1700000000,
            closed_at: None,
            closed_reason: String::new(),
        };
        repo.create_issue(&issue).await.expect("seed issue");

        let label = IssueLabel {
            id: "lbl-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "bug".into(),
            color: "#ff0000".into(),
            created_at: 1700000000,
        };
        repo.create_label(&label).await.expect("seed label");

        let assignment = IssueLabelAssignment {
            id: IssueLabelAssignmentId::from("iss-001:lbl-001"),
            issue_id: "iss-001".into(),
            label_id: "lbl-001".into(),
            created_at: 1700000000,
        };

        repo.assign_label(&assignment).await.expect("assign");
        let labels = repo
            .list_labels_for_issue("iss-001")
            .await
            .expect("list labels for issue");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "bug");

        repo.unassign_label("iss-001", "lbl-001")
            .await
            .expect("unassign");
        let labels2 = repo
            .list_labels_for_issue("iss-001")
            .await
            .expect("list after unassign");
        assert!(labels2.is_empty());
    }
}
