use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

#[tokio::test]
async fn agent_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "agent",
        json!({
            "action": "log_tool",
            "session_id": "00000000-0000-0000-0000-000000000001",
            "data": {
                "tool_name": "search",
                "success": true,
                "duration_ms": 5,
            }
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "agent_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[tokio::test]
async fn agent_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "agent",
        json!({
            "action": 123,
            "session_id": "00000000-0000-0000-0000-000000000001",
            "data": {"tool_name": "search"},
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "agent_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
