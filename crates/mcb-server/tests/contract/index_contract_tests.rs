use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn index_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("index_status", &json!({}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "index_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn index_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("index_repo", &json!({"max_file_size": "not_a_number"}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "index_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
