use serde_json::json;

use crate::common::workspace_root;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

#[tokio::test]
async fn vcs_happy_path_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request(
        "vcs",
        json!({
            "action": "list_repositories",
            "repo_path": workspace_root(),
            "limit": 1,
        }),
    );
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "vcs_happy_path",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}

#[tokio::test]
async fn vcs_invalid_args_contract_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let request = tool_call_request("vcs", json!({"action": 123}));
    let (status, response) = call_tool(&request).await?;

    insta::assert_json_snapshot!(
        "vcs_invalid_args",
        snapshot_payload(&request, status, &response)
    );
    Ok(())
}
