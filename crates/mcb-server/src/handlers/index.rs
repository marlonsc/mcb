//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Index handler for codebase indexing operations.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mcb_domain::ports::IndexingServiceInterface;
use mcb_domain::value_objects::CollectionId;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use crate::args::{IndexAction, IndexArgs};
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::collections::normalize_collection_name;

/// Handler for codebase indexing MCP tool operations.
#[derive(Clone)]
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexHandler {
    /// Create a new `IndexHandler`.
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    fn validate_request(args: &IndexArgs) -> Result<(PathBuf, CollectionId), McpError> {
        let path_str = args
            .path
            .as_ref()
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: path", None))?;
        let path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(McpError::invalid_params(
                "Specified path does not exist",
                None,
            ));
        }
        if !path.is_dir() {
            return Err(McpError::invalid_params(
                "Specified path is not a directory",
                None,
            ));
        }
        let collection_name = args
            .collection
            .as_deref()
            .ok_or_else(|| McpError::invalid_params("collection parameter is required", None))?;
        let collection_id = normalize_collection_name(collection_name)
            .map_err(|reason| McpError::invalid_params(reason, None))?;
        Ok((path, collection_id))
    }

    /// Handle an index tool request.
    ///
    /// # Errors
    /// Returns an error when required arguments are missing or invalid.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        match args.action {
            IndexAction::Start | IndexAction::GitIndex => {
                let (path, collection_id) = Self::validate_request(&args)?;
                let timer = Instant::now();
                match self
                    .indexing_service
                    .index_codebase(&path, &collection_id)
                    .await
                {
                    Ok(result) => Ok(ResponseFormatter::format_indexing_success(
                        &result,
                        &path,
                        timer.elapsed(),
                    )),
                    Err(e) => Ok(to_contextual_tool_error(e)),
                }
            }
            IndexAction::Status => {
                let status = self.indexing_service.get_status();
                Ok(ResponseFormatter::format_indexing_status(&status))
            }
            IndexAction::Clear => {
                let error_path = args
                    .path
                    .as_ref()
                    .map(PathBuf::from)
                    .or_else(|| std::env::current_dir().ok())
                    .ok_or_else(|| {
                        McpError::invalid_params(
                            "path parameter is required (working directory unavailable)",
                            None,
                        )
                    })?;
                let collection_name = args.collection.as_deref().ok_or_else(|| {
                    McpError::invalid_params("collection parameter is required", None)
                })?;
                let milvus_collection = match normalize_collection_name(collection_name) {
                    Ok(id) => id,
                    Err(reason) => {
                        return Err(McpError::invalid_params(reason, None));
                    }
                };
                let milvus_collection_str = milvus_collection.to_string();
                match self
                    .indexing_service
                    .clear_collection(&milvus_collection)
                    .await
                {
                    Ok(()) => Ok(ResponseFormatter::format_clear_index(
                        &milvus_collection_str,
                    )),
                    Err(e) => Ok(ResponseFormatter::format_indexing_error(
                        &format!("Failed to clear collection {milvus_collection_str}: {e}"),
                        &error_path,
                    )),
                }
            }
        }
    }
}
