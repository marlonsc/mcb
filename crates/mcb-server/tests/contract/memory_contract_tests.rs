use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

#[tokio::test]
async fn memory_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "memory",
        json!({
            "action": "list",
            "resource": "observation",
            "limit": 10,
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "memory_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[tokio::test]
async fn memory_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("memory", json!({"action": 123, "resource": "observation"}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "memory_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
