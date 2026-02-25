use crate::utils::test_fixtures::{
    create_test_mcp_server, golden_content_to_string, test_api_key, test_organization, test_user,
};
use mcb_domain::entities::ApiKey;
use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use mcb_server::mcp_server::McpServer;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

fn api_key_from_result(result: &rmcp::model::CallToolResult) -> ApiKey {
    let text = golden_content_to_string(result);
    serde_json::from_str(&text).expect("api key response json")
}

fn api_key_list_from_result(result: &rmcp::model::CallToolResult) -> Vec<ApiKey> {
    let text = golden_content_to_string(result);
    serde_json::from_str(&text).expect("api key list response json")
}

async fn create_org_and_user(server: &McpServer, org_id: &str, email: &str) -> (String, String) {
    let org_h = server.org_entity_handler();

    let org = test_organization(org_id);
    let create_org = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::Org,
            id: None,
            org_id: None,
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!(org)),
        }))
        .await;
    assert!(
        create_org.is_ok(),
        "org create should succeed: {create_org:?}"
    );

    let user = test_user(org_id, email);
    let user_id = user.id.clone();
    let create_user = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::User,
            id: None,
            org_id: Some(org_id.to_string()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!(user)),
        }))
        .await;
    assert!(
        create_user.is_ok(),
        "user create should succeed: {create_user:?}"
    );

    (org_id.to_string(), user_id)
}

async fn create_api_key(server: &McpServer, org_id: &str, user_id: &str, name: &str) -> ApiKey {
    let org_h = server.org_entity_handler();
    let key = test_api_key(user_id, org_id, name);
    let create = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id.to_string()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!(key.clone())),
        }))
        .await;
    assert!(create.is_ok(), "api key create should succeed: {create:?}");
    api_key_from_result(&create.expect("api key create result"))
}

#[tokio::test]
async fn golden_api_key_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-cg", "cg@example.com").await;
    let created_key = create_api_key(&server, &org_id, &user_id, "primary-key").await;

    let org_h = server.org_entity_handler();
    let get = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id.clone()),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(get.is_ok(), "api key get should succeed: {get:?}");

    let fetched_key = api_key_from_result(&get.expect("api key get result"));
    assert_eq!(fetched_key.id, created_key.id);
    assert_eq!(fetched_key.user_id, user_id);
    assert_eq!(fetched_key.org_id, org_id);
    assert_eq!(fetched_key.name, "primary-key");
    assert_eq!(fetched_key.key_hash, created_key.key_hash);
}

#[tokio::test]
async fn golden_api_key_list_by_org() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-list", "list@example.com").await;

    let _ = create_api_key(&server, &org_id, &user_id, "list-key-1").await;
    let _ = create_api_key(&server, &org_id, &user_id, "list-key-2").await;
    let _ = create_api_key(&server, &org_id, &user_id, "list-key-3").await;

    let org_h = server.org_entity_handler();
    let list = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::List,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(list.is_ok(), "api key list should succeed: {list:?}");

    let keys = api_key_list_from_result(&list.expect("api key list result"));
    assert_eq!(keys.len(), 3);
}

#[tokio::test]
async fn golden_api_key_revoke() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-revoke", "revoke@example.com").await;
    let created_key = create_api_key(&server, &org_id, &user_id, "revoke-key").await;
    let revoke_at = 1_700_000_123i64;

    let org_h = server.org_entity_handler();
    let revoke = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Update,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id.clone()),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!({ "revoked_at": revoke_at })),
        }))
        .await;
    assert!(revoke.is_ok(), "api key revoke should succeed: {revoke:?}");

    let get = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id),
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        get.is_ok(),
        "api key get after revoke should succeed: {get:?}"
    );

    let revoked_key = api_key_from_result(&get.expect("api key get after revoke"));
    assert_eq!(revoked_key.revoked_at, Some(revoke_at));
}

#[tokio::test]
async fn golden_api_key_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-delete", "delete@example.com").await;
    let created_key = create_api_key(&server, &org_id, &user_id, "delete-key").await;

    let org_h = server.org_entity_handler();
    let delete = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Delete,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id.clone()),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(delete.is_ok(), "api key delete should succeed: {delete:?}");

    let get_after_delete = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id),
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        get_after_delete.is_err(),
        "deleted api key must not be found"
    );
}

#[tokio::test]
async fn golden_api_key_create_with_scopes() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-scopes", "scopes@example.com").await;
    let mut key = test_api_key(&user_id, &org_id, "scoped-key");
    key.scopes_json = "[\"read\",\"write\"]".to_string();

    let org_h = server.org_entity_handler();
    let create = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!(key)),
        }))
        .await;
    assert!(
        create.is_ok(),
        "api key create with scopes should succeed: {create:?}"
    );
    let created = api_key_from_result(&create.expect("create scoped key"));

    let get = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created.id),
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(get.is_ok(), "api key get should succeed: {get:?}");
    let fetched = api_key_from_result(&get.expect("get scoped key"));
    assert_eq!(fetched.scopes_json, "[\"read\",\"write\"]");
}

#[tokio::test]
async fn golden_api_key_create_with_expiration() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-exp", "exp@example.com").await;
    let mut key = test_api_key(&user_id, &org_id, "expiring-key");
    key.expires_at = Some(1_800_000_000i64);

    let org_h = server.org_entity_handler();
    let create = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!(key)),
        }))
        .await;
    assert!(
        create.is_ok(),
        "api key create with expiration should succeed: {create:?}"
    );
    let created = api_key_from_result(&create.expect("create expiring key"));

    let get = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created.id),
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(get.is_ok(), "api key get should succeed: {get:?}");
    let fetched = api_key_from_result(&get.expect("get expiring key"));
    assert_eq!(fetched.expires_at, Some(1_800_000_000i64));
}

#[tokio::test]
async fn golden_api_key_revoke_sets_timestamp() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-revoke-ts", "revokets@example.com").await;
    let created_key = create_api_key(&server, &org_id, &user_id, "revoke-ts-key").await;
    let revoke_at = 1_700_000_999i64;

    let org_h = server.org_entity_handler();
    let revoke = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Update,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id.clone()),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!({ "revoked_at": revoke_at })),
        }))
        .await;
    assert!(revoke.is_ok(), "api key revoke should succeed: {revoke:?}");

    let get = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Get,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id),
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        get.is_ok(),
        "api key get after revoke should succeed: {get:?}"
    );
    let revoked_key = api_key_from_result(&get.expect("api key after revoke"));
    assert!(revoked_key.revoked_at.unwrap_or(0) > 0);
}

#[tokio::test]
async fn golden_api_key_create_missing_data() {
    let (server, _td) = create_test_mcp_server().await;
    let org_h = server.org_entity_handler();
    let create = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Create,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some("golden-org-key-missing-data".to_string()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(create.is_err(), "create without data must fail");
}

#[tokio::test]
async fn golden_api_key_full_lifecycle() {
    let (server, _td) = create_test_mcp_server().await;
    let (org_id, user_id) =
        create_org_and_user(&server, "golden-org-key-full", "full@example.com").await;
    let created_key = create_api_key(&server, &org_id, &user_id, "full-lifecycle-key").await;
    let org_h = server.org_entity_handler();

    let list_before = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::List,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        list_before.is_ok(),
        "list after create should succeed: {list_before:?}"
    );
    let keys_before = api_key_list_from_result(&list_before.expect("list after create"));
    assert_eq!(keys_before.len(), 1);

    let revoke = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Update,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id.clone()),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: Some(json!({ "revoked_at": 1_700_001_111i64 })),
        }))
        .await;
    assert!(revoke.is_ok(), "revoke should succeed: {revoke:?}");

    let list_after_revoke = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::List,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        list_after_revoke.is_ok(),
        "list after revoke should succeed: {list_after_revoke:?}"
    );
    let keys_after_revoke =
        api_key_list_from_result(&list_after_revoke.expect("list after revoke"));
    assert_eq!(keys_after_revoke.len(), 1);
    assert!(keys_after_revoke[0].revoked_at.unwrap_or(0) > 0);

    let delete = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::Delete,
            resource: OrgEntityResource::ApiKey,
            id: Some(created_key.id),
            org_id: Some(org_id.clone()),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(delete.is_ok(), "delete should succeed: {delete:?}");

    let list_after_delete = org_h
        .handle(Parameters(OrgEntityArgs {
            action: OrgEntityAction::List,
            resource: OrgEntityResource::ApiKey,
            id: None,
            org_id: Some(org_id),
            team_id: None,
            user_id: None,
            email: None,
            data: None,
        }))
        .await;
    assert!(
        list_after_delete.is_ok(),
        "list after delete should succeed: {list_after_delete:?}"
    );
    let keys_after_delete =
        api_key_list_from_result(&list_after_delete.expect("list after delete"));
    assert_eq!(keys_after_delete.len(), 0);
}
