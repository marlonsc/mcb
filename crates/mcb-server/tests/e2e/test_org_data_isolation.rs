use crate::utils::test_fixtures::{
    TEST_ORG_ID_A, TEST_ORG_ID_B, create_test_mcp_server, test_api_key, test_organization,
    test_team, test_user,
};
use crate::utils::text::extract_text;
use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::Value;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn org_args(action: OrgEntityAction, resource: OrgEntityResource) -> OrgEntityArgs {
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

fn parse_list_len(text: &str) -> usize {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.as_array().map(std::vec::Vec::len))
        .unwrap_or(0)
}

async fn create_org(server: &mcb_server::mcp_server::McpServer, org_id: &str) {
    let org = test_organization(org_id);
    let mut args = org_args(OrgEntityAction::Create, OrgEntityResource::Org);
    args.org_id = Some(org_id.to_owned());
    args.data = Some(serde_json::to_value(&org).expect("serialize org"));

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "org create should succeed: {:?}", result);
}

async fn create_user_in_org(
    server: &mcb_server::mcp_server::McpServer,
    org_id: &str,
    email: &str,
) -> String {
    let user = test_user(org_id, email);
    let user_id = user.id.clone();
    let mut args = org_args(OrgEntityAction::Create, OrgEntityResource::User);
    args.org_id = Some(org_id.to_owned());
    args.data = Some(serde_json::to_value(&user).expect("serialize user"));

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "user create should succeed: {:?}", result);
    user_id
}

async fn create_team_in_org(
    server: &mcb_server::mcp_server::McpServer,
    org_id: &str,
    name: &str,
) -> String {
    let team = test_team(org_id, name);
    let team_id = team.id.clone();
    let mut args = org_args(OrgEntityAction::Create, OrgEntityResource::Team);
    args.org_id = Some(org_id.to_owned());
    args.data = Some(serde_json::to_value(&team).expect("serialize team"));

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "team create should succeed: {:?}", result);
    team_id
}

async fn create_api_key_in_org(
    server: &mcb_server::mcp_server::McpServer,
    user_id: &str,
    org_id: &str,
    name: &str,
) -> String {
    let key = test_api_key(user_id, org_id, name);
    let key_id = key.id.clone();
    let mut args = org_args(OrgEntityAction::Create, OrgEntityResource::ApiKey);
    args.org_id = Some(org_id.to_owned());
    args.data = Some(serde_json::to_value(&key).expect("serialize key"));

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(
        result.is_ok(),
        "api key create should succeed: {:?}",
        result
    );
    key_id
}

async fn list_count(
    server: &mcb_server::mcp_server::McpServer,
    resource: OrgEntityResource,
    org_id: &str,
) -> usize {
    let mut args = org_args(OrgEntityAction::List, resource);
    args.org_id = Some(org_id.to_owned());

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    assert!(result.is_ok(), "list should succeed: {:?}", result);
    let text = extract_text(&result.expect("checked ok").content);
    parse_list_len(&text)
}

#[tokio::test]
async fn golden_isolation_users_scoped_to_org() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let _user_id = create_user_in_org(&server, TEST_ORG_ID_A, "isolation-user-a@example.com").await;

    let count_b = list_count(&server, OrgEntityResource::User, TEST_ORG_ID_B).await;
    assert_eq!(count_b, 0, "org-B user list should be empty");
    Ok(())
}

#[tokio::test]
async fn golden_isolation_teams_scoped_to_org() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let _team_id = create_team_in_org(&server, TEST_ORG_ID_A, "platform-a").await;

    let count_b = list_count(&server, OrgEntityResource::Team, TEST_ORG_ID_B).await;
    assert_eq!(count_b, 0, "org-B team list should be empty");
    Ok(())
}

#[tokio::test]
async fn golden_isolation_api_keys_scoped_to_org() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let user_id = create_user_in_org(&server, TEST_ORG_ID_A, "apikey-user-a@example.com").await;
    let _key_id = create_api_key_in_org(&server, &user_id, TEST_ORG_ID_A, "org-a-key").await;

    let count_b = list_count(&server, OrgEntityResource::ApiKey, TEST_ORG_ID_B).await;
    assert_eq!(count_b, 0, "org-B api key list should be empty");
    Ok(())
}

#[tokio::test]
async fn golden_isolation_org_a_invisible_to_org_b() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let user_id = create_user_in_org(&server, TEST_ORG_ID_A, "full-scenario-a@example.com").await;
    let _team_id = create_team_in_org(&server, TEST_ORG_ID_A, "ops-a").await;
    let _key_id =
        create_api_key_in_org(&server, &user_id, TEST_ORG_ID_A, "full-scenario-key").await;

    let users_b = list_count(&server, OrgEntityResource::User, TEST_ORG_ID_B).await;
    let teams_b = list_count(&server, OrgEntityResource::Team, TEST_ORG_ID_B).await;
    let keys_b = list_count(&server, OrgEntityResource::ApiKey, TEST_ORG_ID_B).await;

    assert_eq!(users_b, 0, "org-B user list should be empty");
    assert_eq!(teams_b, 0, "org-B team list should be empty");
    assert_eq!(keys_b, 0, "org-B api key list should be empty");
    Ok(())
}

#[tokio::test]
async fn golden_isolation_cross_org_get_fails() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let user_id = create_user_in_org(&server, TEST_ORG_ID_A, "cross-get-a@example.com").await;

    let mut args = org_args(OrgEntityAction::Get, OrgEntityResource::User);
    args.id = Some(user_id);
    args.org_id = Some(TEST_ORG_ID_B.to_owned());

    let result = server.org_entity_handler().handle(Parameters(args)).await;
    match result {
        Err(_) => {}
        Ok(resp) => {
            let text = extract_text(&resp.content);
            let parsed = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
            let is_empty = parsed.is_null()
                || parsed.as_array().is_some_and(std::vec::Vec::is_empty)
                || parsed
                    .as_object()
                    .is_some_and(serde_json::Map::<String, Value>::is_empty);

            assert!(
                is_empty,
                "cross-org get should return empty or error, got: {}",
                text
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn golden_isolation_both_orgs_coexist() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;
    create_org(&server, TEST_ORG_ID_A).await;
    create_org(&server, TEST_ORG_ID_B).await;

    let _a1 = create_user_in_org(&server, TEST_ORG_ID_A, "coexist-a1@example.com").await;
    let _a2 = create_user_in_org(&server, TEST_ORG_ID_A, "coexist-a2@example.com").await;
    let _b1 = create_user_in_org(&server, TEST_ORG_ID_B, "coexist-b1@example.com").await;

    let count_a = list_count(&server, OrgEntityResource::User, TEST_ORG_ID_A).await;
    let count_b = list_count(&server, OrgEntityResource::User, TEST_ORG_ID_B).await;

    assert_eq!(count_a, 2, "org-A should list only org-A users");
    assert_eq!(count_b, 1, "org-B should list only org-B users");
    Ok(())
}
