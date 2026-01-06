//! MCP Server implementation

use crate::core::error::{Error, Result};
use crate::services::{ContextService, IndexingService, SearchService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// MCP tool response
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolResponse {
    pub content: Vec<CallToolResultContent>,
    pub is_error: bool,
}

/// MCP tool result content
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// MCP tool definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP tool handlers - simplified for MVP
pub struct McpToolHandlers {
    indexing_service: Arc<IndexingService>,
    search_service: Arc<SearchService>,
}

impl McpToolHandlers {
    /// Create new MCP tool handlers
    pub fn new() -> Result<Self> {
        // For MVP, create services directly
        let service_provider = crate::factory::ServiceProvider::new();
        let context_service = Arc::new(ContextService::new(&service_provider)?);
        let indexing_service = Arc::new(IndexingService::new(context_service.clone()));
        let search_service = Arc::new(SearchService::new(context_service));

        Ok(Self {
            indexing_service,
            search_service,
        })
    }

    /// Get the list of available tools
    pub fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "index_codebase".to_string(),
                description: "Index a codebase directory for semantic search".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the codebase directory to index"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "search_code".to_string(),
                description: "Search for code using natural language queries".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query to search for"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
        ]
    }

    /// Handle tool calls
    pub async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResponse> {
        match name {
            "index_codebase" => self.handle_index_codebase(arguments).await,
            "search_code" => self.handle_search_code(arguments).await,
            _ => Err(Error::invalid_argument(format!("Unknown tool: {}", name))),
        }
    }

    async fn handle_index_codebase(&self, args: serde_json::Value) -> Result<CallToolResponse> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_argument("Missing path argument"))?;

        let path = std::path::Path::new(path);
        let collection = "default";

        match self.indexing_service.index_directory(path, collection).await {
            Ok(chunk_count) => {
                let message = format!(
                    "✅ Successfully indexed {} code chunks from '{}'",
                    chunk_count, path.display()
                );

                Ok(CallToolResponse {
                    content: vec![CallToolResultContent::Text { text: message }],
                    is_error: false,
                })
            }
            Err(e) => {
                let error_msg = format!("❌ Failed to index codebase: {}", e);

                Ok(CallToolResponse {
                    content: vec![CallToolResultContent::Text { text: error_msg }],
                    is_error: true,
                })
            }
        }
    }

    async fn handle_search_code(&self, args: serde_json::Value) -> Result<CallToolResponse> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_argument("Missing query argument"))?;

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let collection = "default";

        match self.search_service.search(collection, query, limit).await {
            Ok(results) => {
                let mut message = format!("🔍 **Search Results for:** \"{}\"\n\n", query);

                if results.is_empty() {
                    message.push_str("❌ No relevant results found.");
                } else {
                    message.push_str(&format!("📊 **Found {} results:**\n\n", results.len()));

                    for (i, result) in results.iter().enumerate() {
                        message.push_str(&format!(
                            "**{}. {}** (line {})\n```\n{}\n```\n*Score: {:.3}*\n\n",
                            i + 1,
                            result.file_path,
                            result.line_number,
                            result.content.chars().take(100).collect::<String>(),
                            result.score
                        ));
                    }
                }

                Ok(CallToolResponse {
                    content: vec![CallToolResultContent::Text { text: message }],
                    is_error: false,
                })
            }
            Err(e) => {
                let error_msg = format!("❌ Search failed: {}", e);

                Ok(CallToolResponse {
                    content: vec![CallToolResultContent::Text { text: error_msg }],
                    is_error: true,
                })
            }
        }
    }
}