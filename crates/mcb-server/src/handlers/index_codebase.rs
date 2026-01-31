//! Index Codebase Tool Handler
//!
//! Handles the index_codebase MCP tool call using the domain indexing service.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_application::domain_services::search::IndexingServiceInterface;

use crate::args::IndexCodebaseArgs;
use crate::collection_mapping::map_collection_name;
use crate::formatter::ResponseFormatter;

/// Validated indexing request ready for processing
struct ValidatedRequest {
    path: PathBuf,
    milvus_collection: String,
}

/// Handler for codebase indexing operations
#[derive(Clone)]
pub struct IndexCodebaseHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexCodebaseHandler {
    /// Create a new index_codebase handler
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    /// Validate arguments and resolve collection name
    fn validate_request(args: &IndexCodebaseArgs) -> Result<ValidatedRequest, CallToolResult> {
        let path = PathBuf::from(&args.path);
        if !path.exists() {
            return Err(ResponseFormatter::format_indexing_error(
                "Specified path does not exist",
                &path,
            ));
        }
        if !path.is_dir() {
            return Err(ResponseFormatter::format_indexing_error(
                "Specified path is not a directory",
                &path,
            ));
        }
        let collection_name = args.collection.as_deref().unwrap_or("default");
        let milvus_collection = match map_collection_name(collection_name) {
            Ok(name) => name,
            Err(e) => {
                return Err(ResponseFormatter::format_indexing_error(
                    &format!("Failed to map collection name '{}': {}", collection_name, e),
                    &path,
                ));
            }
        };
        Ok(ValidatedRequest {
            path,
            milvus_collection,
        })
    }

    /// Handle the index_codebase tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexCodebaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let request = match Self::validate_request(&args) {
            Ok(req) => req,
            Err(error_result) => return Ok(error_result),
        };

        let timer = Instant::now();
        match self
            .indexing_service
            .index_codebase(&request.path, &request.milvus_collection)
            .await
        {
            Ok(result) => Ok(ResponseFormatter::format_indexing_success(
                &result,
                &request.path,
                timer.elapsed(),
            )),
            Err(e) => Ok(ResponseFormatter::format_indexing_error(
                &e.to_string(),
                &request.path,
            )),
        }
    }
}
