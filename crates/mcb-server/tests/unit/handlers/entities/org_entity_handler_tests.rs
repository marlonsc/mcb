//! Org entity CRUD: organization, user, and team lifecycle operations.

use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use mcb_server::handlers::entities::OrgEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use rstest::rstest;

fn handler() -> TestResult<OrgEntityHandler> {
    let state = crate::utils::test_fixtures::shared_mcb_state()?;
    Ok(OrgEntityHandler::new(
        state.mcp_server.org_entity_repository(),
    ))
}

fn org_args(action: OrgEntityAction, resource: OrgEntityResource) -> OrgEntityArgs {
    OrgEntityArgs {
        action,
        resource,
        id: None,
        email: None,
        org_id: None,
        team_id: None,
        user_id: None,
        data: None,
    }
}

fn org_payload(id: &str, org_id: &str) -> serde_json::Value {
    json!({
        "id": id, "org_id": org_id, "name": format!("Org {id}"),
        "slug": format!("org-{id}"), "settings_json": "{}",
        "created_at": 1, "updated_at": 1
    })
}

async fn list_count(h: &OrgEntityHandler) -> usize {
    let args = org_args(OrgEntityAction::List, OrgEntityResource::Org);
    let content = h
        .handle(Parameters(args))
        .await
        .ok()
        .map(|r| r.content)
        .unwrap_or_default();
    let text = extract_text_from(&content);
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.as_array().map(Vec::len))
        .unwrap_or(0)
}

#[rstest]
#[tokio::test]
async fn get_user_requires_id_or_email() -> TestResult {
    let h = handler()?;
    let err = h
        .handle(Parameters(org_args(
            OrgEntityAction::Get,
            OrgEntityResource::User,
        )))
        .await
        .expect_err("must reject missing id/email");
    assert!(err.message.contains("id or email required for user get"));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn conflicting_org_id_rejected_without_side_effect() -> TestResult {
    let h = handler()?;
    let before = list_count(&h).await;

    let mut args = org_args(OrgEntityAction::Create, OrgEntityResource::Org);
    args.org_id = Some("org-a".to_owned());
    args.data = Some(org_payload("org-conflict", "org-b"));

    let err = h
        .handle(Parameters(args))
        .await
        .expect_err("conflicting org_id must fail");
    assert!(err.message.contains("conflicting org_id"));
    assert_eq!(list_count(&h).await, before);
    Ok(())
}
