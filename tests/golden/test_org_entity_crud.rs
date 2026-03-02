use mcb_domain::utils::tests::fixtures::*;
use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

fn base_args(action: OrgEntityAction, resource: OrgEntityResource) -> OrgEntityArgs {
    OrgEntityArgs {
        action,
        resource,
        id: None,
        org_id: None,
        team_id: None,
        user_id: None,
        email: None,
        data: None,
    }
}

fn result_json(res: &rmcp::model::CallToolResult) -> serde_json::Value {
    let text = golden_content_to_string(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("response should be valid JSON: {text}; error: {e}"))
}

async fn create_org(server: &mcb_server::mcp_server::McpServer, org_id: &str) {
    let org = test_organization(org_id);
    let payload = serde_json::to_value(&org).expect("serialize org payload");

    let mut args = base_args(OrgEntityAction::Create, OrgEntityResource::Org);
    args.org_id = Some(org.id.clone());
    args.data = Some(payload);

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "org create should succeed: {result:?}");
}

async fn create_user(
    server: &mcb_server::mcp_server::McpServer,
    org_id: &str,
    email: &str,
) -> serde_json::Value {
    let user = test_user(org_id, email);
    let payload = serde_json::to_value(&user).expect("serialize user payload");

    let mut args = base_args(OrgEntityAction::Create, OrgEntityResource::User);
    args.org_id = Some(org_id.to_string());
    args.data = Some(payload);

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "user create should succeed: {result:?}");
    result_json(&result.expect("user create response"))
}

async fn create_team(
    server: &mcb_server::mcp_server::McpServer,
    org_id: &str,
    name: &str,
) -> serde_json::Value {
    let team = test_team(org_id, name);
    let payload = serde_json::to_value(&team).expect("serialize team payload");

    let mut args = base_args(OrgEntityAction::Create, OrgEntityResource::Team);
    args.org_id = Some(org_id.to_string());
    args.data = Some(payload);

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "team create should succeed: {result:?}");
    result_json(&result.expect("team create response"))
}

#[tokio::test]
async fn golden_org_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org = test_organization("golden-org-create-get");
    let payload = serde_json::to_value(&org).expect("serialize org payload");

    let mut create_args = base_args(OrgEntityAction::Create, OrgEntityResource::Org);
    create_args.org_id = Some(org.id.clone());
    create_args.data = Some(payload);

    let create_result = server
        .org_entity_handler()
        .handle(Parameters(create_args))
        .await;
    assert!(
        create_result.is_ok(),
        "org create should succeed: {create_result:?}"
    );

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::Org);
    get_args.id = Some(org.id.clone());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_ok(), "org get should succeed: {get_result:?}");

    let body = result_json(&get_result.expect("org get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(org.id.as_str())
    );
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some(org.name.as_str())
    );
}

#[tokio::test]
async fn golden_org_list() {
    let (server, _td) = create_test_mcp_server().await;

    create_org(&server, "golden-org-list-1").await;
    create_org(&server, "golden-org-list-2").await;

    let list_args = base_args(OrgEntityAction::List, OrgEntityResource::Org);
    let list_result = server
        .org_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "org list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("org list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "org list should have at least 2 results, got {count}"
    );
}

#[tokio::test]
async fn golden_org_update() {
    let (server, _td) = create_test_mcp_server().await;
    let mut org = test_organization("golden-org-update");
    create_org(&server, &org.id).await;

    org.name = "Golden Org Updated".to_string();
    let mut update_args = base_args(OrgEntityAction::Update, OrgEntityResource::Org);
    update_args.data = Some(serde_json::to_value(&org).expect("serialize org update payload"));

    let update_result = server
        .org_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "org update should succeed: {update_result:?}"
    );

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::Org);
    get_args.id = Some(org.id.clone());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "org get should succeed after update: {get_result:?}"
    );

    let body = result_json(&get_result.expect("org get after update response"));
    assert_eq!(
        body.get("name").and_then(serde_json::Value::as_str),
        Some("Golden Org Updated")
    );
}

#[tokio::test]
async fn golden_org_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org = test_organization("golden-org-delete");
    create_org(&server, &org.id).await;

    let mut delete_args = base_args(OrgEntityAction::Delete, OrgEntityResource::Org);
    delete_args.id = Some(org.id.clone());
    let delete_result = server
        .org_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "org delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::Org);
    get_args.id = Some(org.id);
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "org get should fail after delete");
}

#[tokio::test]
async fn golden_org_create_missing_data() {
    let (server, _td) = create_test_mcp_server().await;

    let args = base_args(OrgEntityAction::Create, OrgEntityResource::Org);
    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "org create without data should fail");
}

#[tokio::test]
async fn golden_user_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-user-create-get-org";
    create_org(&server, org_id).await;

    let user_body = create_user(&server, org_id, "golden-user-create-get@example.com").await;
    let user_id = user_body
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!user_id.is_empty(), "created user id must be present");

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::User);
    get_args.id = Some(user_id.clone());
    get_args.org_id = Some(org_id.to_string());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "user get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("user get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(user_id.as_str())
    );
    assert_eq!(
        body.get("email").and_then(serde_json::Value::as_str),
        Some("golden-user-create-get@example.com")
    );
}

#[tokio::test]
async fn golden_user_get_by_email() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-user-get-by-email-org";
    let email = "golden-user-get-by-email@example.com";
    create_org(&server, org_id).await;

    let created = create_user(&server, org_id, email).await;
    let created_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!created_id.is_empty(), "created user id must be present");

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::User);
    get_args.org_id = Some(org_id.to_string());
    get_args.email = Some(email.to_string());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "user get by email should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("user get by email response"));
    assert_eq!(
        body.get("email").and_then(serde_json::Value::as_str),
        Some(email)
    );
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(created_id.as_str())
    );
}

#[tokio::test]
async fn golden_user_list_by_org() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-user-list-org";
    create_org(&server, org_id).await;

    let _ = create_user(&server, org_id, "golden-user-list-1@example.com").await;

    let admin_user = test_admin_user(org_id, "golden-user-list-admin@example.com");
    let mut create_admin_args = base_args(OrgEntityAction::Create, OrgEntityResource::User);
    create_admin_args.org_id = Some(org_id.to_string());
    create_admin_args.data =
        Some(serde_json::to_value(&admin_user).expect("serialize admin user payload"));
    let create_admin_result = server
        .org_entity_handler()
        .handle(Parameters(create_admin_args))
        .await;
    assert!(
        create_admin_result.is_ok(),
        "admin user create should succeed: {create_admin_result:?}"
    );

    let mut list_args = base_args(OrgEntityAction::List, OrgEntityResource::User);
    list_args.org_id = Some(org_id.to_string());
    let list_result = server
        .org_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "user list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("user list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "user list should have at least 2 users, got {count}"
    );
}

#[tokio::test]
async fn golden_user_update() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-user-update-org";
    create_org(&server, org_id).await;

    let created = create_user(&server, org_id, "golden-user-update@example.com").await;
    let mut updated = created.clone();
    updated["display_name"] = json!("Golden User Updated");

    let mut update_args = base_args(OrgEntityAction::Update, OrgEntityResource::User);
    update_args.org_id = Some(org_id.to_string());
    update_args.data = Some(updated);
    let update_result = server
        .org_entity_handler()
        .handle(Parameters(update_args))
        .await;
    assert!(
        update_result.is_ok(),
        "user update should succeed: {update_result:?}"
    );

    let user_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!user_id.is_empty(), "created user id must be present");

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::User);
    get_args.id = Some(user_id);
    get_args.org_id = Some(org_id.to_string());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "user get should succeed after update: {get_result:?}"
    );

    let body = result_json(&get_result.expect("user get response after update"));
    assert_eq!(
        body.get("display_name").and_then(serde_json::Value::as_str),
        Some("Golden User Updated")
    );
}

#[tokio::test]
async fn golden_user_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-user-delete-org";
    create_org(&server, org_id).await;

    let created = create_user(&server, org_id, "golden-user-delete@example.com").await;
    let user_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!user_id.is_empty(), "created user id must be present");

    let mut delete_args = base_args(OrgEntityAction::Delete, OrgEntityResource::User);
    delete_args.id = Some(user_id.clone());
    let delete_result = server
        .org_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "user delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::User);
    get_args.id = Some(user_id);
    get_args.org_id = Some(org_id.to_string());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "user get should fail after delete");
}

#[tokio::test]
async fn golden_user_create_missing_data() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(OrgEntityAction::Create, OrgEntityResource::User);
    args.org_id = Some("golden-user-create-missing-data-org".to_string());
    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "user create without data should fail");
}

#[tokio::test]
async fn golden_user_get_missing_id_and_email() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(OrgEntityAction::Get, OrgEntityResource::User);
    args.org_id = Some("golden-user-get-missing-org".to_string());
    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "user get without id/email should fail");

    let err = result.expect_err("missing user id/email should return error");
    assert!(
        err.message.contains("id or email required for user get"),
        "unexpected error: {}",
        err.message
    );
}

#[tokio::test]
async fn golden_team_create_and_get() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-team-create-get-org";
    create_org(&server, org_id).await;

    let created = create_team(&server, org_id, "Golden Team Create Get").await;
    let team_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!team_id.is_empty(), "created team id must be present");

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::Team);
    get_args.id = Some(team_id.clone());
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(
        get_result.is_ok(),
        "team get should succeed: {get_result:?}"
    );

    let body = result_json(&get_result.expect("team get response"));
    assert_eq!(
        body.get("id").and_then(serde_json::Value::as_str),
        Some(team_id.as_str())
    );
}

#[tokio::test]
async fn golden_team_list() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-team-list-org";
    create_org(&server, org_id).await;

    let _ = create_team(&server, org_id, "Golden Team List 1").await;
    let _ = create_team(&server, org_id, "Golden Team List 2").await;

    let mut list_args = base_args(OrgEntityAction::List, OrgEntityResource::Team);
    list_args.org_id = Some(org_id.to_string());
    let list_result = server
        .org_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "team list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("team list response"));
    let count = body.as_array().map(std::vec::Vec::len).unwrap_or(0);
    assert!(
        count >= 2,
        "team list should have at least 2 teams, got {count}"
    );
}

#[tokio::test]
async fn golden_team_delete() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-team-delete-org";
    create_org(&server, org_id).await;

    let created = create_team(&server, org_id, "Golden Team Delete").await;
    let team_id = created
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!team_id.is_empty(), "created team id must be present");

    let mut delete_args = base_args(OrgEntityAction::Delete, OrgEntityResource::Team);
    delete_args.id = Some(team_id.clone());
    let delete_result = server
        .org_entity_handler()
        .handle(Parameters(delete_args))
        .await;
    assert!(
        delete_result.is_ok(),
        "team delete should succeed: {delete_result:?}"
    );

    let mut get_args = base_args(OrgEntityAction::Get, OrgEntityResource::Team);
    get_args.id = Some(team_id);
    let get_result = server
        .org_entity_handler()
        .handle(Parameters(get_args))
        .await;
    assert!(get_result.is_err(), "team get should fail after delete");
}

#[tokio::test]
async fn golden_team_update_unsupported() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(OrgEntityAction::Update, OrgEntityResource::Team);
    args.data = Some(json!({"id": "unused", "name": "Unsupported Update"}));
    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "team update should be unsupported");
}

#[tokio::test]
async fn golden_team_member_add_and_list() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-team-member-add-list-org";
    create_org(&server, org_id).await;

    let user = create_user(&server, org_id, "golden-team-member-add-list@example.com").await;
    let user_id = user
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!user_id.is_empty(), "created user id must be present");

    let team = create_team(&server, org_id, "Golden Team Member Add").await;
    let team_id = team
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!team_id.is_empty(), "created team id must be present");

    let member = test_team_member(&team_id, &user_id);
    let mut add_args = base_args(OrgEntityAction::Create, OrgEntityResource::TeamMember);
    add_args.data = Some(serde_json::to_value(&member).expect("serialize team member payload"));
    let add_result = server
        .org_entity_handler()
        .handle(Parameters(add_args))
        .await;
    assert!(
        add_result.is_ok(),
        "team member add should succeed: {add_result:?}"
    );

    let mut list_args = base_args(OrgEntityAction::List, OrgEntityResource::TeamMember);
    list_args.team_id = Some(team_id);
    let list_result = server
        .org_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "team member list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("team member list response"));
    let members = body.as_array().cloned().unwrap_or_default();
    let has_member = members.iter().any(|entry| {
        entry.get("user_id").and_then(serde_json::Value::as_str) == Some(user_id.as_str())
    });
    assert!(
        has_member,
        "team member list should include newly added member"
    );
}

#[tokio::test]
async fn golden_team_member_remove() {
    let (server, _td) = create_test_mcp_server().await;
    let org_id = "golden-team-member-remove-org";
    create_org(&server, org_id).await;

    let user = create_user(&server, org_id, "golden-team-member-remove@example.com").await;
    let user_id = user
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!user_id.is_empty(), "created user id must be present");

    let team = create_team(&server, org_id, "Golden Team Member Remove").await;
    let team_id = team
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    assert!(!team_id.is_empty(), "created team id must be present");

    let member = test_team_member(&team_id, &user_id);
    let mut add_args = base_args(OrgEntityAction::Create, OrgEntityResource::TeamMember);
    add_args.data = Some(serde_json::to_value(&member).expect("serialize team member payload"));
    let add_result = server
        .org_entity_handler()
        .handle(Parameters(add_args))
        .await;
    assert!(
        add_result.is_ok(),
        "team member add should succeed: {add_result:?}"
    );

    let mut remove_args = base_args(OrgEntityAction::Delete, OrgEntityResource::TeamMember);
    remove_args.team_id = Some(team_id.clone());
    remove_args.user_id = Some(user_id.clone());
    let remove_result = server
        .org_entity_handler()
        .handle(Parameters(remove_args))
        .await;
    assert!(
        remove_result.is_ok(),
        "team member remove should succeed: {remove_result:?}"
    );

    let mut list_args = base_args(OrgEntityAction::List, OrgEntityResource::TeamMember);
    list_args.team_id = Some(team_id);
    let list_result = server
        .org_entity_handler()
        .handle(Parameters(list_args))
        .await;
    assert!(
        list_result.is_ok(),
        "team member list should succeed: {list_result:?}"
    );

    let body = result_json(&list_result.expect("team member list response"));
    let members = body.as_array().cloned().unwrap_or_default();
    let has_member = members.iter().any(|entry| {
        entry.get("user_id").and_then(serde_json::Value::as_str) == Some(user_id.as_str())
    });
    assert!(!has_member, "team member should be removed from list");
}

#[tokio::test]
async fn golden_team_member_get_unsupported() {
    let (server, _td) = create_test_mcp_server().await;

    let mut args = base_args(OrgEntityAction::Get, OrgEntityResource::TeamMember);
    args.id = Some("non-existent-team-member".to_string());
    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_err(), "team member get should be unsupported");
}
