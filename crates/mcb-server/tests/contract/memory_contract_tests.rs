use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn memory_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "list_memories",
        &json!({
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

#[rstest]
#[tokio::test]
async fn memory_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("list_memories", &json!({"limit": "not_a_number"}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "memory_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn memory_get_observation_ids_none_contract_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("get_memories", &json!({}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "memory_get_observation_ids_none",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn memory_get_observation_ids_empty_contract_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "get_memories",
        &json!({
            "ids": []
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "memory_get_observation_ids_empty",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
