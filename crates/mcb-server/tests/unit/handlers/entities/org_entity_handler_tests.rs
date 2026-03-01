use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use mcb_server::handlers::entities::OrgEntityHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::utils::text::extract_text;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn create_handler() -> TestResult<OrgEntityHandler> {
    let state = crate::utils::shared_context::shared_mcb_state()?;
    Ok(OrgEntityHandler::new(
        state.mcp_server.org_entity_repository(),
    ))
}

#[tokio::test]
async fn get_user_requires_id_or_email() -> TestResult {
    let handler = create_handler()?;
    let args = OrgEntityArgs {
        action: OrgEntityAction::Get,
        resource: OrgEntityResource::User,
        id: None,
        email: None,
        org_id: None,
        team_id: None,
        user_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing id/email");
    assert!(err.message.contains("id or email required for user get"));
    Ok(())
}

fn org_payload(id: &str, org_id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "org_id": org_id,
        "name": format!("Org {id}"),
        "slug": format!("org-{id}"),
        "settings_json": "{}",
        "created_at": 1,
        "updated_at": 1
    })
}

async fn list_org_count(handler: &OrgEntityHandler) -> usize {
    let list_args = OrgEntityArgs {
        action: OrgEntityAction::List,
        resource: OrgEntityResource::Org,
        id: None,
        email: None,
        org_id: None,
        team_id: None,
        user_id: None,
        data: None,
    };
    let content = handler
        .handle(Parameters(list_args))
        .await
        .ok()
        .map(|r| r.content)
        .unwrap_or_default();
    let text = extract_text(&content);
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.as_array().map(std::vec::Vec::len))
        .unwrap_or(0)
}

#[tokio::test]
async fn create_org_with_conflicting_org_id_rejected_without_side_effect() -> TestResult {
    let handler = create_handler()?;
    let before_count = list_org_count(&handler).await;

    let create_args = OrgEntityArgs {
        action: OrgEntityAction::Create,
        resource: OrgEntityResource::Org,
        id: None,
        email: None,
        org_id: Some("org-a".to_owned()),
        team_id: None,
        user_id: None,
        data: Some(org_payload("org-create-conflict", "org-b")),
    };

    let err = handler
        .handle(Parameters(create_args))
        .await
        .expect_err("conflicting org_id must fail");
    assert!(err.message.contains("conflicting org_id"));

    let after_count = list_org_count(&handler).await;
    assert_eq!(after_count, before_count);
    Ok(())
}
