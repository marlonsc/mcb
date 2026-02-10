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

    fn validate_request(args: &IndexArgs) -> Result<(PathBuf, CollectionId), CallToolResult> {
        let missing_path_err = || {
            CallToolResult::error(vec![rmcp::model::Content::text(
                "Missing required parameter: path",
            )])
        };
        let path_str = args.path.as_ref().ok_or_else(missing_path_err)?;
        let path = PathBuf::from(path_str);
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
        let collection_id = match normalize_collection_name(collection_name) {
            Ok(id) => id,
            Err(e) => {
                let _ = e;
                return Err(ResponseFormatter::format_indexing_error(
                    &format!("Failed to map collection name '{}'", collection_name),
                    &path,
                ));
            }
        };
        Ok((path, collection_id))
    }

    /// Handle an index tool request.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        match args.action {
            IndexAction::Start => {
                let (path, collection_id) = match Self::validate_request(&args) {
                    Ok(value) => value,
                    Err(error_result) => return Ok(error_result),
                };
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
                        let _ = e;
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
                let collection_name = args.collection.as_deref().unwrap_or("default");
                let milvus_collection = match normalize_collection_name(collection_name) {
                    Ok(id) => id,
                    Err(e) => {
                        let _ = e;
                        return Ok(ResponseFormatter::format_indexing_error(
                            &format!("Failed to map collection name '{}'", collection_name),
                            &PathBuf::from("."),
                        ));
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
