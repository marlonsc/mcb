use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn session_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("list_sessions", &json!({"limit": 10}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "session_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn session_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("list_sessions", &json!({"limit": "not_a_number"}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "session_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
