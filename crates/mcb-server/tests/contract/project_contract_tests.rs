use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

#[tokio::test]
async fn project_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "project",
        json!({
            "action": "list",
            "resource": "project",
            "project_id": "project-contract",
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "project_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[tokio::test]
async fn project_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "project",
        json!({
            "action": 123,
            "resource": "project",
            "project_id": "project-contract",
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "project_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
