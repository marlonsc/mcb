use std::sync::Arc;

use mcb_domain::error::Error;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_providers::database::create_memory_repository_with_executor;

pub const TEST_NOW: i64 = 1_000_000;

pub async fn setup_executor() -> (Arc<dyn DatabaseExecutor>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");
    (executor, temp_dir)
}

pub async fn seed_org(executor: &dyn DatabaseExecutor, org_id: &str, name: &str, slug: &str) {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(name.to_string()),
                SqlParam::String(slug.to_string()),
                SqlParam::String("{}".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed org");
}

pub async fn seed_project(
    executor: &dyn DatabaseExecutor,
    project_id: &str,
    org_id: &str,
    name: &str,
    path: &str,
) {
    executor
        .execute(
            "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_string()),
                SqlParam::String(org_id.to_string()),
                SqlParam::String(name.to_string()),
                SqlParam::String(path.to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed project");
}

pub async fn seed_user(
    executor: &dyn DatabaseExecutor,
    user_id: &str,
    org_id: &str,
    email: &str,
    display_name: &str,
) {
    executor
        .execute(
            "INSERT INTO users (id, org_id, email, display_name, role, api_key_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(user_id.to_string()),
                SqlParam::String(org_id.to_string()),
                SqlParam::String(email.to_string()),
                SqlParam::String(display_name.to_string()),
                SqlParam::String("admin".to_string()),
                SqlParam::Null,
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed user");
}

pub async fn seed_default_scope(executor: &dyn DatabaseExecutor) {
    seed_org(
        executor,
        mcb_domain::constants::keys::DEFAULT_ORG_ID,
        "default",
        "default",
    )
    .await;
    seed_project(
        executor,
        "proj-1",
        mcb_domain::constants::keys::DEFAULT_ORG_ID,
        "Test Project",
        "/test",
    )
    .await;
    seed_user(
        executor,
        "user-1",
        mcb_domain::constants::keys::DEFAULT_ORG_ID,
        "test@example.com",
        "Test User",
    )
    .await;
}

pub async fn seed_isolated_org_scope(executor: &dyn DatabaseExecutor, org_id: &str) {
    seed_org(executor, org_id, org_id, org_id).await;
    seed_project(
        executor,
        &format!("proj-{org_id}"),
        org_id,
        &format!("Project {org_id}"),
        &format!("/{org_id}"),
    )
    .await;
    seed_user(
        executor,
        &format!("user-{org_id}"),
        org_id,
        &format!("{org_id}@test.com"),
        &format!("User {org_id}"),
    )
    .await;
}

pub fn assert_not_found<T>(result: mcb_domain::error::Result<T>) {
    assert!(matches!(result, Err(Error::NotFound { .. })));
}
