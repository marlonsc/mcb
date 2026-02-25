//! Integration tests for the unified `SeaORM` Entity CRUD Repository.
//!
//! Run with: `cargo test -p mcb-providers --test entity_repo`
//!
//! These tests exercise the `SeaOrmEntityRepository` against an in-memory `SQLite`
//! database with full migrations applied, covering all 16 entity types:
//!
//! VCS: `repository`, `branch`, `worktree`, `assignment`
//! Org: `organization`, `user`, `team`, `team_member`, `api_key`
//! Plan: `plan`, `plan_version`, `plan_review`
//! Issue: `project_issue`, `issue_comment`, `issue_label`, `issue_label_assignment`

#![allow(clippy::expect_used, missing_docs)]

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm_migration::MigratorTrait;

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::entities::team::{Team, TeamMember, TeamMemberRole};
use mcb_domain::entities::user::{User, UserRole};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use mcb_domain::entities::{ApiKey, Organization};
use mcb_domain::ports::{
    ApiKeyRegistry, IssueCommentRegistry, IssueLabelAssignmentManager, IssueLabelRegistry,
    IssueRegistry, OrgRegistry, PlanRegistry, PlanReviewRegistry, PlanVersionRegistry,
    TeamMemberManager, TeamRegistry, UserRegistry, VcsEntityRepository,
};
use mcb_domain::value_objects::ids::{IssueLabelAssignmentId, TeamMemberId};

use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::repos::entity::SeaOrmEntityRepository;

async fn setup_db() -> Arc<DatabaseConnection> {
    let db = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory SQLite");
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "PRAGMA foreign_keys = ON;".to_string(),
    ))
    .await
    .expect("enable foreign keys");
    Migrator::up(&db, None).await.expect("migration up");
    Arc::new(db)
}

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

async fn seed_project(repo: &SeaOrmEntityRepository) {
    use mcb_providers::database::seaorm::entities::project;
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let proj = project::ActiveModel {
        id: ActiveValue::Set("proj-001".into()),
        org_id: ActiveValue::Set("org-001".into()),
        name: ActiveValue::Set("Test Project".into()),
        path: ActiveValue::Set("/tmp/proj".into()),
        created_at: ActiveValue::Set(1700000000),
        updated_at: ActiveValue::Set(1700000000),
    };
    proj.insert(repo.db()).await.expect("seed project");
}

// ======================================================================
// VCS: Repository
// ======================================================================

#[tokio::test]
async fn vcs_repository_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_org(&repo).await;
    seed_project(&repo).await;

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

    repo.create_repository(&r).await.expect("create");
    let got = repo
        .get_repository("org-001", "repo-001")
        .await
        .expect("get");
    assert_eq!(got.name, "mcb");
    assert_eq!(got.vcs_type, VcsType::Git);

    let list = repo
        .list_repositories("org-001", "proj-001")
        .await
        .expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = r.clone();
    updated.name = "mcb-updated".into();
    repo.update_repository(&updated).await.expect("update");
    let got2 = repo
        .get_repository("org-001", "repo-001")
        .await
        .expect("get updated");
    assert_eq!(got2.name, "mcb-updated");

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
// VCS: Branch
// ======================================================================

#[tokio::test]
async fn vcs_branch_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_org(&repo).await;
    seed_project(&repo).await;

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
    let got = repo.get_branch("br-001").await.expect("get");
    assert_eq!(got.name, "main");
    assert!(got.is_default);

    let list = repo.list_branches("repo-001").await.expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = b.clone();
    updated.head_commit = "def456".into();
    repo.update_branch(&updated).await.expect("update");
    let got2 = repo.get_branch("br-001").await.expect("get updated");
    assert_eq!(got2.head_commit, "def456");

    repo.delete_branch("br-001").await.expect("delete");
    let list2 = repo
        .list_branches("repo-001")
        .await
        .expect("list after delete");
    assert!(list2.is_empty());
}

// ======================================================================
// VCS: Worktree
// ======================================================================

#[tokio::test]
async fn vcs_worktree_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_org(&repo).await;
    seed_project(&repo).await;

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
    let got2 = repo.get_worktree("wt-001").await.expect("get updated");
    assert_eq!(got2.status, WorktreeStatus::InUse);

    repo.delete_worktree("wt-001").await.expect("delete");
    let list2 = repo
        .list_worktrees("repo-001")
        .await
        .expect("list after delete");
    assert!(list2.is_empty());
}

// ======================================================================
// VCS: Assignment
// ======================================================================

#[tokio::test]
async fn vcs_assignment_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_org(&repo).await;
    seed_project(&repo).await;

    // Seed repo + branch + worktree + agent session
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
    use mcb_providers::database::seaorm::entities::agent_session;
    use sea_orm::{ActiveModelTrait, ActiveValue};
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
    ses.insert(repo.db()).await.expect("seed agent session");

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
// Org: Organization
// ======================================================================

#[tokio::test]
async fn org_organization_crud() {
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
    let got2 = repo.get_org("org-001").await.expect("get updated");
    assert_eq!(got2.name, "Acme Updated");

    repo.delete_org("org-001").await.expect("delete");
    let list2 = repo.list_orgs().await.expect("list after delete");
    assert!(list2.is_empty());
}

// ======================================================================
// Org: User
// ======================================================================

#[tokio::test]
async fn org_user_crud() {
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
    let got = repo.get_user("usr-001").await.expect("get");
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
    let got2 = repo.get_user("usr-001").await.expect("get updated");
    assert_eq!(got2.display_name, "Alice Updated");

    repo.delete_user("usr-001").await.expect("delete");
    let list2 = repo.list_users("org-001").await.expect("list after delete");
    assert!(list2.is_empty());
}

// ======================================================================
// Org: Team
// ======================================================================

#[tokio::test]
async fn org_team_crud() {
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
// Org: TeamMember
// ======================================================================

#[tokio::test]
async fn org_team_member_crud() {
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
// Org: ApiKey
// ======================================================================

#[tokio::test]
async fn org_api_key_crud() {
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
// Plan: Plan
// ======================================================================

#[tokio::test]
async fn plan_plan_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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
        .expect("get updated");
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
// Plan: PlanVersion
// ======================================================================

#[tokio::test]
async fn plan_version_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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
// Plan: PlanReview
// ======================================================================

#[tokio::test]
async fn plan_review_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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

    // Seed reviewer
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
// Issue: ProjectIssue
// ======================================================================

#[tokio::test]
async fn issue_project_issue_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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
        .expect("get updated");
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
// Issue: IssueComment
// ======================================================================

#[tokio::test]
async fn issue_comment_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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
// Issue: IssueLabel
// ======================================================================

#[tokio::test]
async fn issue_label_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_org(&repo).await;
    seed_project(&repo).await;

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
// Issue: IssueLabelAssignment
// ======================================================================

#[tokio::test]
async fn issue_label_assignment_crud() {
    let db = setup_db().await;
    let repo = SeaOrmEntityRepository::new(db);
    seed_user(&repo).await;
    seed_project(&repo).await;

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
