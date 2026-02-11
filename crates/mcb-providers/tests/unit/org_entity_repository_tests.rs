use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::{
    ApiKey, Organization, Team, TeamMember, TeamMemberRole, User, UserRole,
};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::OrgEntityRepository;
use mcb_providers::database::{SqliteOrgEntityRepository, create_memory_repository_with_executor};

async fn setup_repo() -> (
    SqliteOrgEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");
    let repo = SqliteOrgEntityRepository::new(Arc::clone(&executor));
    (repo, executor, temp_dir)
}

fn create_test_org(id: &str, name: &str, slug: &str) -> Organization {
    let now = 1_000_000_i64;
    Organization {
        id: id.to_string(),
        name: name.to_string(),
        slug: slug.to_string(),
        settings_json: "{}".to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn create_test_user(id: &str, org_id: &str, email: &str) -> User {
    let now = 1_000_000_i64;
    User {
        id: id.to_string(),
        org_id: org_id.to_string(),
        email: email.to_string(),
        display_name: format!("User {id}"),
        role: UserRole::Member,
        api_key_hash: None,
        created_at: now,
        updated_at: now,
    }
}

fn create_test_team(id: &str, org_id: &str, name: &str) -> Team {
    let now = 1_000_000_i64;
    Team {
        id: id.to_string(),
        org_id: org_id.to_string(),
        name: name.to_string(),
        created_at: now,
    }
}

fn create_test_api_key(id: &str, user_id: &str, org_id: &str, name: &str) -> ApiKey {
    let now = 1_000_000_i64;
    ApiKey {
        id: id.to_string(),
        user_id: user_id.to_string(),
        org_id: org_id.to_string(),
        key_hash: format!("hash-{id}"),
        name: name.to_string(),
        scopes_json: "[]".to_string(),
        expires_at: None,
        created_at: now,
        revoked_at: None,
    }
}

#[tokio::test]
async fn test_org_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org("org-1", "Test Org", "test-org");

    repo.create_org(&org).await.expect("create");

    let retrieved = repo.get_org("org-1").await.expect("get");
    assert_eq!(retrieved.name, "Test Org");
    assert_eq!(retrieved.slug, "test-org");

    let list = repo.list_orgs().await.expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = org.clone();
    updated.name = "Updated Org".to_string();
    updated.updated_at = 2_000_000;
    repo.update_org(&updated).await.expect("update");

    let after_update = repo.get_org("org-1").await.expect("get");
    assert_eq!(after_update.name, "Updated Org");

    repo.delete_org("org-1").await.expect("delete");
    assert!(repo.get_org("org-1").await.is_err());
}

#[tokio::test]
async fn test_user_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await.expect("create org");

    let user = create_test_user("user-1", DEFAULT_ORG_ID, "user1@test.com");
    repo.create_user(&user).await.expect("create user");

    let retrieved = repo.get_user("user-1").await.expect("get");
    assert_eq!(retrieved.email, "user1@test.com");
    assert_eq!(retrieved.role, UserRole::Member);

    let list = repo.list_users(DEFAULT_ORG_ID).await.expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = user.clone();
    updated.display_name = "Updated User".to_string();
    updated.role = UserRole::Admin;
    updated.updated_at = 2_000_000;
    repo.update_user(&updated).await.expect("update");

    let after_update = repo.get_user("user-1").await.expect("get");
    assert_eq!(after_update.display_name, "Updated User");
    assert_eq!(after_update.role, UserRole::Admin);

    repo.delete_user("user-1").await.expect("delete");
    assert!(repo.get_user("user-1").await.is_err());
}

#[tokio::test]
async fn test_get_user_by_email() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await.expect("create org");

    let user = create_test_user("user-1", DEFAULT_ORG_ID, "alice@example.com");
    repo.create_user(&user).await.expect("create user");

    let found = repo
        .get_user_by_email(DEFAULT_ORG_ID, "alice@example.com")
        .await
        .expect("get by email");
    assert_eq!(found.id, "user-1");

    assert!(
        repo.get_user_by_email(DEFAULT_ORG_ID, "nobody@example.com")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_team_and_members() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await.expect("create org");

    let u1 = create_test_user("user-1", DEFAULT_ORG_ID, "u1@test.com");
    let u2 = create_test_user("user-2", DEFAULT_ORG_ID, "u2@test.com");
    repo.create_user(&u1).await.expect("create u1");
    repo.create_user(&u2).await.expect("create u2");

    let team = create_test_team("team-1", DEFAULT_ORG_ID, "Backend Team");
    repo.create_team(&team).await.expect("create team");

    let retrieved = repo.get_team("team-1").await.expect("get");
    assert_eq!(retrieved.name, "Backend Team");

    let teams = repo.list_teams(DEFAULT_ORG_ID).await.expect("list teams");
    assert_eq!(teams.len(), 1);

    let now = 1_000_000_i64;
    let m1 = TeamMember {
        team_id: "team-1".to_string(),
        user_id: "user-1".to_string(),
        role: TeamMemberRole::Lead,
        joined_at: now,
    };
    let m2 = TeamMember {
        team_id: "team-1".to_string(),
        user_id: "user-2".to_string(),
        role: TeamMemberRole::Member,
        joined_at: now,
    };
    repo.add_team_member(&m1).await.expect("add m1");
    repo.add_team_member(&m2).await.expect("add m2");

    let members = repo
        .list_team_members("team-1")
        .await
        .expect("list members");
    assert_eq!(members.len(), 2);

    repo.remove_team_member("team-1", "user-1")
        .await
        .expect("remove m1");
    let after_remove = repo.list_team_members("team-1").await.expect("list");
    assert_eq!(after_remove.len(), 1);
    assert_eq!(after_remove[0].user_id, "user-2");

    repo.remove_team_member("team-1", "user-2")
        .await
        .expect("remove m2");

    repo.delete_team("team-1").await.expect("delete team");
    assert!(repo.get_team("team-1").await.is_err());
}

#[tokio::test]
async fn test_api_key_lifecycle() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await.expect("create org");
    let user = create_test_user("user-1", DEFAULT_ORG_ID, "u@test.com");
    repo.create_user(&user).await.expect("create user");

    let key = create_test_api_key("key-1", "user-1", DEFAULT_ORG_ID, "CI Key");
    repo.create_api_key(&key).await.expect("create key");

    let retrieved = repo.get_api_key("key-1").await.expect("get");
    assert_eq!(retrieved.name, "CI Key");
    assert!(retrieved.revoked_at.is_none());

    let keys = repo.list_api_keys(DEFAULT_ORG_ID).await.expect("list");
    assert_eq!(keys.len(), 1);

    repo.revoke_api_key("key-1", 2_000_000)
        .await
        .expect("revoke");
    let after_revoke = repo.get_api_key("key-1").await.expect("get");
    assert_eq!(after_revoke.revoked_at, Some(2_000_000));

    repo.delete_api_key("key-1").await.expect("delete");
    assert!(repo.get_api_key("key-1").await.is_err());
}

#[tokio::test]
async fn test_org_isolation_users() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org_a = create_test_org("org-A", "Org A", "org-a");
    let org_b = create_test_org("org-B", "Org B", "org-b");
    repo.create_org(&org_a).await.expect("create org-A");
    repo.create_org(&org_b).await.expect("create org-B");

    let user = create_test_user("user-1", "org-A", "alice@a.com");
    repo.create_user(&user).await.expect("create user");

    let users_a = repo.list_users("org-A").await.expect("list org-A");
    assert_eq!(users_a.len(), 1);

    let users_b = repo.list_users("org-B").await.expect("list org-B");
    assert!(users_b.is_empty());
}

#[tokio::test]
async fn test_delete_org_fk_constraint() {
    let (repo, _executor, _temp) = setup_repo().await;
    let org = create_test_org("org-fk", "FK Org", "fk-org");
    repo.create_org(&org).await.expect("create org");

    let user = create_test_user("user-fk", "org-fk", "fk@test.com");
    repo.create_user(&user).await.expect("create user");

    let result = repo.delete_org("org-fk").await;
    assert!(
        result.is_err(),
        "Deleting org with users should fail due to FK constraint"
    );
}
