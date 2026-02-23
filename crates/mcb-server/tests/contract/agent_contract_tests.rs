use serde_json::json;

use crate::common::{call_tool, snapshot_payload, tool_call_request};

fn normalize_tool_call_ids(mut payload: serde_json::Value) -> serde_json::Value {
    if let Some(text) = payload
        .pointer_mut("/response/result/content/0/text")
        .and_then(|v| v.as_str().map(String::from))
    {
        let normalized = if let Some(start) = text.find("tc_") {
            let end = start + 39; // "tc_" (3) + UUID (36) = 39
            if end <= text.len() {
                format!(
                    "{}tc_00000000-0000-0000-0000-000000000000{}",
                    &text[..start],
                    &text[end..]
                )
            } else {
                text
            }
        } else {
            text
        };
        payload["response"]["result"]["content"][0]["text"] = serde_json::Value::String(normalized);
    }
    payload
}

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
        normalize_tool_call_ids(snapshot_payload(&request, status, &response))
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
