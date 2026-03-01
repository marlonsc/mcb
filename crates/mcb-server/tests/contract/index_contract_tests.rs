use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

#[tokio::test]
async fn index_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("index", &json!({"action": "status"}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "index_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[tokio::test]
async fn index_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("index", &json!({"action": 123}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "index_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
