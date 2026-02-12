//! Index handler for codebase indexing operations.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mcb_domain::ports::services::IndexingServiceInterface;
use mcb_domain::value_objects::CollectionId;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use crate::args::{IndexAction, IndexArgs};
use crate::formatter::ResponseFormatter;
use crate::utils::collections::normalize_collection_name;

/// Handler for codebase indexing MCP tool operations.
#[derive(Clone)]
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexHandler {
    /// Create a new IndexHandler.
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
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        match args.action {
            IndexAction::Start => {
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
                    Err(e) => {
                        tracing::warn!(error = %e, path = ?path, "indexing failed");
                        Ok(ResponseFormatter::format_indexing_error(
                            "Indexing failed",
                            &path,
                        ))
                    }
                }
            }
            IndexAction::Status => {
                let status = self.indexing_service.get_status();
                Ok(ResponseFormatter::format_indexing_status(&status))
            }
            IndexAction::Clear => {
                let collection_name = args.collection.as_deref().ok_or_else(|| {
                    McpError::invalid_params("collection parameter is required", None)
                })?;
                let milvus_collection = match normalize_collection_name(collection_name) {
                    Ok(id) => id,
                    Err(reason) => {
                        return Err(McpError::invalid_params(reason, None));
                    }
                };
                match self
                    .indexing_service
                    .clear_collection(&milvus_collection)
                    .await
                {
                    Ok(()) => Ok(ResponseFormatter::format_clear_index(
                        milvus_collection.as_str(),
                    )),
                    Err(e) => Ok(ResponseFormatter::format_indexing_error(
                        &format!(
                            "Failed to clear collection {}: {}",
                            milvus_collection.as_str(),
                            e
                        ),
                        &PathBuf::from("."),
                    )),
                }
            }
        }
    }
}
