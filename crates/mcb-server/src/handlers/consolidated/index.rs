//! Index handler for codebase indexing operations.

use crate::args::{IndexAction, IndexArgs};
use crate::collection_mapping::map_collection_name;
use crate::formatter::ResponseFormatter;
use mcb_domain::ports::services::IndexingServiceInterface;
use mcb_domain::value_objects::CollectionId;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

/// Handler for codebase indexing MCP tool operations.
#[derive(Clone)]
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexHandler {
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    fn validate_request(args: &IndexArgs) -> Result<(PathBuf, CollectionId), CallToolResult> {
        let path_str = args.path.as_ref().ok_or_else(|| {
            CallToolResult::error(vec![rmcp::model::Content::text(
                "Missing required parameter: path",
            )])
        })?;
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
        let milvus_collection = match map_collection_name(collection_name) {
            Ok(name) => name,
            Err(e) => {
                return Err(ResponseFormatter::format_indexing_error(
                    &format!("Failed to map collection name '{}': {}", collection_name, e),
                    &path,
                ));
            }
        };
        Ok((path, CollectionId::new(milvus_collection)))
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            IndexAction::Start => {
                let (path, milvus_collection) = match Self::validate_request(&args) {
                    Ok(value) => value,
                    Err(error_result) => return Ok(error_result),
                };
                let timer = Instant::now();
                match self
                    .indexing_service
                    .index_codebase(&path, &milvus_collection)
                    .await
                {
                    Ok(result) => Ok(ResponseFormatter::format_indexing_success(
                        &result,
                        &path,
                        timer.elapsed(),
                    )),
                    Err(e) => Ok(ResponseFormatter::format_indexing_error(
                        &format!("Indexing failed: {}", e),
                        &path,
                    )),
                }
            }
            IndexAction::Status => {
                let status = self.indexing_service.get_status();
                Ok(ResponseFormatter::format_indexing_status(&status))
            }
            IndexAction::Clear => {
                let collection_name = args.collection.as_deref().unwrap_or("default");
                let milvus_collection = match map_collection_name(collection_name) {
                    Ok(name) => name,
                    Err(e) => {
                        return Ok(ResponseFormatter::format_indexing_error(
                            &format!("Failed to map collection name '{}': {}", collection_name, e),
                            &PathBuf::from("."),
                        ));
                    }
                };
                let collection_id = CollectionId::new(milvus_collection.clone());
                match self.indexing_service.clear_collection(&collection_id).await {
                    Ok(()) => Ok(ResponseFormatter::format_clear_index(&milvus_collection)),
                    Err(e) => Ok(ResponseFormatter::format_indexing_error(
                        &format!("Failed to clear collection {}: {}", milvus_collection, e),
                        &PathBuf::from("."),
                    )),
                }
            }
        }
    }
}
