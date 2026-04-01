use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn search_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "search_code",
        &json!({
            "query": "test query",
            "limit": 10,
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "search_happy_path",
        snapshot_payload(&request, status, &response),
        {
            ".response.result.content[0].text" => "[search-text]"
        }
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn search_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "search_code",
        &json!({
            "query": 999,
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "search_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
