//! MCP Server implementation

use crate::core::error::{Error, Result};
use crate::factory::ProviderFactory;
use crate::metrics::PERFORMANCE_METRICS;
use crate::services::{ContextService, IndexingService, SearchService};
use crate::sync::SyncManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

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

/// MCP tool handlers with real providers and metrics
pub struct McpToolHandlers {
    indexing_service: Arc<IndexingService>,
    search_service: Arc<SearchService>,
}

impl McpToolHandlers {
    /// Create new MCP tool handlers with configured providers
    pub fn new() -> Result<Self> {
        Self::with_config(None)
    }

    /// Create handlers with sync manager for coordination
    pub fn with_sync_manager(sync_manager: Arc<SyncManager>) -> Result<Self> {
        Self::with_config(Some(sync_manager))
    }

    /// Internal constructor with optional sync manager
    fn with_config(sync_manager: Option<Arc<SyncManager>>) -> Result<Self> {
        // Load configuration from environment
        let config = crate::config::Config::from_env()
            .map_err(|e| Error::config(format!("Failed to load configuration: {}", e)))?;

        // Create context service with real providers
        let context_service = Arc::new(ProviderFactory::create_context_service(&config)?);

        // Create indexing service with sync coordination
        let indexing_service = Arc::new(if let Some(sync_mgr) = sync_manager {
            IndexingService::with_sync_manager(context_service.clone(), sync_mgr)?
        } else {
            IndexingService::new(context_service.clone())?
        });

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
        let start_time = Instant::now();

        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_argument("Missing path argument"))?;

        let path = std::path::Path::new(path);
        let collection = "default";

        let result = self
            .indexing_service
            .index_directory(path, collection)
            .await;

        let duration = start_time.elapsed();
        let latency_ms = duration.as_millis() as f64;
        let success = result.is_ok();

        // Record performance metrics
        PERFORMANCE_METRICS.record_query(latency_ms, success);

        match result {
            Ok(chunk_count) => {
                let message = format!(
                    "✅ Successfully indexed {} code chunks from '{}' in {:.2}s",
                    chunk_count,
                    path.display(),
                    duration.as_secs_f64()
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
        let start_time = Instant::now();

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_argument("Missing query argument"))?;

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let collection = "default";

        let result = self.search_service.search(collection, query, limit).await;

        let duration = start_time.elapsed();
        let latency_ms = duration.as_millis() as f64;
        let success = result.is_ok();

        // Record performance metrics
        PERFORMANCE_METRICS.record_query(latency_ms, success);

        match result {
            Ok(results) => {
                let mut message = format!("🔍 **Search Results for:** \"{}\"\n\n", query);
                message.push_str(&format!(
                    "⚡ **Search completed in:** {:.2}s\n\n",
                    duration.as_secs_f64()
                ));

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
                            result.content.chars().take(150).collect::<String>(),
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
