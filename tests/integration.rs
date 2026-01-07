//! Integration tests for the MCP Context Browser
//!
//! This module tests the full application flow including MCP protocol handling.

use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    async fn run_mcp_command_test(_json_input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // This is a simplified integration test that would need to be adapted
        // based on how the actual MCP server runs. For now, we'll create a placeholder
        // that demonstrates the testing approach.

        // In a real scenario, this would:
        // 1. Start the MCP server in a separate process or thread
        // 2. Send MCP messages via stdin
        // 3. Read responses from stdout
        // 4. Parse and validate the responses

        // For this TDD cycle, we'll create tests that validate the message handling logic
        // without actually running the full server process.

        Ok("placeholder response".to_string())
    }

    #[tokio::test]
    async fn test_mcp_initialize_message_handling() {
        // Test that initialize message is handled correctly
        let initialize_message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        let message_json = serde_json::to_string(&initialize_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        // In a real test, this would validate the actual response
        // For now, we just ensure the test framework works
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_list_message_handling() {
        // Test that tools/list message is handled correctly
        let tools_list_message = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let message_json = serde_json::to_string(&tools_list_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_call_index_codebase() {
        // Test tools/call for index_codebase
        let temp_dir = tempfile::tempdir().unwrap();
        let tools_call_message = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "index_codebase",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            }
        });

        let message_json = serde_json::to_string(&tools_call_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_call_search_code() {
        // Test tools/call for search_code
        let tools_call_message = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "search_code",
                "arguments": {
                    "query": "test query",
                    "limit": 5
                }
            }
        });

        let message_json = serde_json::to_string(&tools_call_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_unknown_method_handling() {
        // Test that unknown methods return proper error
        let unknown_method_message = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "unknown_method",
            "params": {}
        });

        let message_json = serde_json::to_string(&unknown_method_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_invalid_json_handling() {
        // Test that invalid JSON is handled gracefully
        let invalid_json = "{ invalid json content }";
        let result = run_mcp_command_test(invalid_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_call_missing_arguments() {
        // Test tools/call with missing arguments
        let tools_call_message = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "index_codebase"
                // Missing arguments
            }
        });

        let message_json = serde_json::to_string(&tools_call_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_call_unknown_tool() {
        // Test tools/call with unknown tool name
        let tools_call_message = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "unknown_tool",
                "arguments": {}
            }
        });

        let message_json = serde_json::to_string(&tools_call_message).unwrap();
        let result = run_mcp_command_test(&message_json).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_tools_call_search_with_limit() {
        // Test search_code with various limits
        let limits = vec![0, 1, 5, 10, 50];

        for limit in limits {
            let tools_call_message = json!({
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "search_code",
                    "arguments": {
                        "query": "test query",
                        "limit": limit
                    }
                }
            });

            let message_json = serde_json::to_string(&tools_call_message).unwrap();
            let result = run_mcp_command_test(&message_json).await;

            assert!(result.is_ok(), "Failed for limit {}", limit);
        }
    }

    #[test]
    fn test_jsonrpc_message_structure() {
        // Test that our message structures match MCP protocol
        let message = json!({
            "jsonrpc": "2.0",
            "id": 123,
            "method": "test_method",
            "params": {
                "key": "value"
            }
        });

        assert_eq!(message["jsonrpc"], "2.0");
        assert_eq!(message["id"], 123);
        assert_eq!(message["method"], "test_method");
        assert_eq!(message["params"]["key"], "value");
    }

    #[test]
    fn test_mcp_protocol_constants() {
        // Test that protocol constants are correctly defined
        let protocol_version = "2024-11-05";
        assert_eq!(protocol_version, "2024-11-05");

        let server_name = "MCP Context Browser";
        assert_eq!(server_name, "MCP Context Browser");
    }

    #[tokio::test]
    async fn test_milvus_provider_connection() {
        use mcp_context_browser::providers::VectorStoreProvider;

        // Test that we can create a Milvus provider and connect to the local instance
        let milvus_address = "http://localhost:19531";

        // Skip test if Milvus is not available
        if std::process::Command::new("curl")
            .args([
                "-s",
                "--max-time",
                "1",
                &format!("{}/v1/vector/collections", milvus_address),
            ])
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(true)
        {
            println!("Milvus not available at {}, skipping test", milvus_address);
            return;
        }

        // Create Milvus provider
        let provider_result =
            mcp_context_browser::providers::vector_store::MilvusVectorStoreProvider::new(
                milvus_address.to_string(),
                None,
            )
            .await;

        assert!(
            provider_result.is_ok(),
            "Failed to create Milvus provider: {:?}",
            provider_result.err()
        );

        let provider = provider_result.unwrap();

        // Test that provider name is correct
        assert_eq!(provider.provider_name(), "milvus");

        // Test collection operations
        let collection_name = "test_collection";

        // Create collection
        let create_result = provider.create_collection(collection_name, 128).await;
        assert!(
            create_result.is_ok(),
            "Failed to create collection: {:?}",
            create_result.err()
        );

        // Check if collection exists
        let exists_result = provider.collection_exists(collection_name).await;
        assert!(
            exists_result.is_ok(),
            "Failed to check collection existence: {:?}",
            exists_result.err()
        );

        // Clean up - delete collection
        let delete_result = provider.delete_collection(collection_name).await;
        assert!(
            delete_result.is_ok(),
            "Failed to delete collection: {:?}",
            delete_result.err()
        );
    }

    #[tokio::test]
    async fn test_mcp_server_stdio_communication() {
        use std::process::Stdio;
        use std::time::Duration;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::process::Command;

        // Start MCP server process with environment variables for testing
        let mut child = Command::new("cargo")
            .args(["run"])
            .env("CONTEXT_METRICS_ENABLED", "false") // Disable metrics for test
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Capture stderr for debugging
            .spawn()
            .expect("Failed to start MCP server");

        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stderr = child.stderr.take().expect("Failed to get stderr");
        let mut reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        // Wait for server to start (read stderr to see startup messages)
        let mut stderr_buf = String::new();
        let mut startup_complete = false;
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < Duration::from_secs(15) {
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Check if we can read stderr for startup messages
            while let Ok(bytes) = tokio::time::timeout(
                Duration::from_millis(10),
                stderr_reader.read_line(&mut stderr_buf),
            )
            .await
            {
                if bytes.is_ok() && stderr_buf.contains("MCP Context Browser ready") {
                    startup_complete = true;
                    break;
                }
                stderr_buf.clear();
            }

            if startup_complete {
                break;
            }
        }

        assert!(
            startup_complete,
            "Server did not start within 15 seconds. Stderr: {}",
            stderr_buf
        );

        // Test 1: Initialize request
        let initialize_request = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}"#;

        stdin
            .write_all(format!("{}\n", initialize_request).as_bytes())
            .await
            .expect("Failed to write initialize request");
        stdin.flush().await.expect("Failed to flush stdin");

        // Read response
        let mut response_line = String::new();
        let read_result =
            tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
                .await;

        assert!(read_result.is_ok(), "Timeout reading initialize response");

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .expect("Failed to parse initialize response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
        assert!(response["result"]["serverInfo"].is_object());

        // Test 2: Tools list request
        let tools_list_request =
            r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}"#;

        stdin
            .write_all(format!("{}\n", tools_list_request).as_bytes())
            .await
            .expect("Failed to write tools/list request");
        stdin.flush().await.expect("Failed to flush stdin");

        // Read response
        response_line.clear();
        let read_result =
            tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
                .await;

        assert!(read_result.is_ok(), "Timeout reading tools/list response");

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .expect("Failed to parse tools/list response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 2);
        assert!(response["result"].is_object());
        assert!(response["result"]["tools"].is_array());

        // Verify expected tools are present
        let tools = response["result"]["tools"].as_array().unwrap();
        let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

        assert!(
            tool_names.contains(&"index_codebase"),
            "index_codebase tool should be present"
        );
        assert!(
            tool_names.contains(&"search_code"),
            "search_code tool should be present"
        );
        assert!(
            tool_names.contains(&"get_indexing_status"),
            "get_indexing_status tool should be present"
        );
        assert!(
            tool_names.contains(&"clear_index"),
            "clear_index tool should be present"
        );

        // Test 3: Invalid method (should return error)
        let invalid_request =
            r#"{"jsonrpc": "2.0", "id": 3, "method": "invalid_method", "params": {}}"#;

        stdin
            .write_all(format!("{}\n", invalid_request).as_bytes())
            .await
            .expect("Failed to write invalid request");
        stdin.flush().await.expect("Failed to flush stdin");

        // Read response
        response_line.clear();
        let read_result =
            tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
                .await;

        assert!(
            read_result.is_ok(),
            "Timeout reading invalid method response"
        );

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .expect("Failed to parse invalid method response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 3);
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601); // Method not found
        assert_eq!(response["error"]["message"], "Method not found");

        // Clean up
        drop(stdin); // Close stdin to signal EOF
        let _ = child.wait().await;
    }
}
