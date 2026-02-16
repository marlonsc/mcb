use rstest::rstest;
use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::{
    ApiKey, Organization, Team, TeamMember, TeamMemberRole, User, UserRole,
};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::org_entity_repository::{
    ApiKeyRegistry, OrgRegistry, TeamMemberManager, TeamRegistry, UserRegistry,
};
use mcb_providers::database::SqliteOrgEntityRepository;

use crate::common::entity_test_utils::{TEST_NOW, TestResult, assert_not_found, setup_executor};

async fn setup_repo() -> TestResult<(
    SqliteOrgEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
)> {
    let (executor, temp_dir) = setup_executor().await?;
    let repo = SqliteOrgEntityRepository::new(Arc::clone(&executor));
    Ok((repo, executor, temp_dir))
}

fn create_test_org(id: &str, name: &str, slug: &str) -> Organization {
    Organization {
        id: id.to_owned(),
        name: name.to_owned(),
        slug: slug.to_owned(),
        settings_json: "{}".to_owned(),
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
    }
}

fn create_test_user(id: &str, org_id: &str, email: &str) -> User {
    User {
        metadata: mcb_domain::entities::EntityMetadata {
            id: id.to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        org_id: org_id.to_owned(),
        email: email.to_owned(),
        display_name: format!("User {id}"),
        role: UserRole::Member,
        api_key_hash: None,
    }
}

fn create_test_team(id: &str, org_id: &str, name: &str) -> Team {
    Team {
        id: id.to_owned(),
        org_id: org_id.to_owned(),
        name: name.to_owned(),
        created_at: TEST_NOW,
    }
}

fn create_test_api_key(id: &str, user_id: &str, org_id: &str, name: &str) -> ApiKey {
    ApiKey {
        id: id.to_owned(),
        user_id: user_id.to_owned(),
        org_id: org_id.to_owned(),
        key_hash: format!("hash-{id}"),
        name: name.to_owned(),
        scopes_json: "[]".to_owned(),
        expires_at: None,
        created_at: TEST_NOW,
        revoked_at: None,
    }
}

#[tokio::test]
async fn test_org_crud() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org("org-1", "Test Org", "test-org");

    repo.create_org(&org).await?;

    let retrieved = repo.get_org("org-1").await?;
    assert_eq!(retrieved.name, "Test Org");
    assert_eq!(retrieved.slug, "test-org");

    let list = repo.list_orgs().await?;
    assert_eq!(list.len(), 1);

    let mut updated = org.clone();
    updated.name = "Updated Org".to_owned();
    updated.updated_at = 2_000_000;
    repo.update_org(&updated).await?;

    let after_update = repo.get_org("org-1").await?;
    assert_eq!(after_update.name, "Updated Org");

    repo.delete_org("org-1").await?;
    assert_not_found(&repo.get_org("org-1").await);
    Ok(())
}

#[tokio::test]
async fn test_user_crud() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await?;

    let user = create_test_user("user-1", DEFAULT_ORG_ID, "user1@test.com");
    repo.create_user(&user).await?;

    let retrieved = repo.get_user("user-1").await?;
    assert_eq!(retrieved.email, "user1@test.com");
    assert_eq!(retrieved.role, UserRole::Member);

    let list = repo.list_users(DEFAULT_ORG_ID).await?;
    assert_eq!(list.len(), 1);

    let mut updated = user.clone();
    updated.display_name = "Updated User".to_owned();
    updated.role = UserRole::Admin;
    updated.metadata.updated_at = 2_000_000;
    repo.update_user(&updated).await?;

    let after_update = repo.get_user("user-1").await?;
    assert_eq!(after_update.display_name, "Updated User");
    assert_eq!(after_update.role, UserRole::Admin);

    repo.delete_user("user-1").await?;
    assert_not_found(&repo.get_user("user-1").await);
    Ok(())
}

#[tokio::test]
async fn test_get_user_by_email() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await?;

    let user = create_test_user("user-1", DEFAULT_ORG_ID, "alice@example.com");
    repo.create_user(&user).await?;

    let found = repo
        .get_user_by_email(DEFAULT_ORG_ID, "alice@example.com")
        .await?;
    assert_eq!(found.metadata.id, "user-1");

    assert_not_found(
        &repo
            .get_user_by_email(DEFAULT_ORG_ID, "nobody@example.com")
            .await,
    );
    Ok(())
}

#[tokio::test]
async fn test_team_and_members() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await?;

    let u1 = create_test_user("user-1", DEFAULT_ORG_ID, "u1@test.com");
    let u2 = create_test_user("user-2", DEFAULT_ORG_ID, "u2@test.com");
    repo.create_user(&u1).await?;
    repo.create_user(&u2).await?;

    let team = create_test_team("team-1", DEFAULT_ORG_ID, "Backend Team");
    repo.create_team(&team).await?;

    let retrieved = repo.get_team("team-1").await?;
    assert_eq!(retrieved.name, "Backend Team");

    let teams = repo.list_teams(DEFAULT_ORG_ID).await?;
    assert_eq!(teams.len(), 1);

    use mcb_domain::utils::id;
    use mcb_domain::value_objects::ids::TeamMemberId;

    let m1 = TeamMember {
        id: TeamMemberId::from_uuid(id::deterministic("team_member", "team-1:user-1")),
        team_id: "team-1".to_owned(),
        user_id: "user-1".to_owned(),
        role: TeamMemberRole::Lead,
        joined_at: TEST_NOW,
    };
    let m2 = TeamMember {
        id: TeamMemberId::from_uuid(id::deterministic("team_member", "team-1:user-2")),
        team_id: "team-1".to_owned(),
        user_id: "user-2".to_owned(),
        role: TeamMemberRole::Member,
        joined_at: TEST_NOW,
    };
    repo.add_team_member(&m1).await?;
    repo.add_team_member(&m2).await?;

    let members = repo.list_team_members("team-1").await?;
    assert_eq!(members.len(), 2);

    repo.remove_team_member("team-1", "user-1").await?;
    let after_remove = repo.list_team_members("team-1").await?;
    assert_eq!(after_remove.len(), 1);
    assert_eq!(after_remove[0].user_id, "user-2");

    repo.remove_team_member("team-1", "user-2").await?;

    repo.delete_team("team-1").await?;
    assert_not_found(&repo.get_team("team-1").await);
    Ok(())
}

#[tokio::test]
async fn test_api_key_lifecycle() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org(DEFAULT_ORG_ID, "Default", "default");
    repo.create_org(&org).await?;
    let user = create_test_user("user-1", DEFAULT_ORG_ID, "u@test.com");
    repo.create_user(&user).await?;

    let key = create_test_api_key("key-1", "user-1", DEFAULT_ORG_ID, "CI Key");
    repo.create_api_key(&key).await?;

    let retrieved = repo.get_api_key("key-1").await?;
    assert_eq!(retrieved.name, "CI Key");
    assert!(retrieved.revoked_at.is_none());

    let keys = repo.list_api_keys(DEFAULT_ORG_ID).await?;
    assert_eq!(keys.len(), 1);

    repo.revoke_api_key("key-1", 2_000_000).await?;
    let after_revoke = repo.get_api_key("key-1").await?;
    assert_eq!(after_revoke.revoked_at, Some(2_000_000));

    repo.delete_api_key("key-1").await?;
    assert_not_found(&repo.get_api_key("key-1").await);
    Ok(())
}

#[rstest]
#[case("org-A", 1)]
#[case("org-B", 0)]
#[tokio::test]
async fn org_isolation_users(#[case] org_id: &str, #[case] expected_count: usize) -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org_a = create_test_org("org-A", "Org A", "org-a");
    let org_b = create_test_org("org-B", "Org B", "org-b");
    repo.create_org(&org_a).await?;
    repo.create_org(&org_b).await?;

    let user = create_test_user("user-1", "org-A", "alice@a.com");
    repo.create_user(&user).await?;

    let users = repo.list_users(org_id).await?;
    assert_eq!(users.len(), expected_count);
    Ok(())
}

#[tokio::test]
async fn test_delete_org_fk_constraint() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let org = create_test_org("org-fk", "FK Org", "fk-org");
    repo.create_org(&org).await?;

    let user = create_test_user("user-fk", "org-fk", "fk@test.com");
    repo.create_user(&user).await?;

    let result = repo.delete_org("org-fk").await;
    assert!(
        result.is_err(),
        "Deleting org with users should fail due to FK constraint"
    );
    Ok(())
}
