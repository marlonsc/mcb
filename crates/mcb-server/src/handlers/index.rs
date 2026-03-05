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

handler_new!(IndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
});

impl IndexHandler {
    fn validate_request(args: &IndexArgs) -> Result<(PathBuf, CollectionId), McpError> {
        let path = args
            .path
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| {
                McpError::invalid_params("path is required (working directory unavailable)", None)
            })?;
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
        let collection_id = match args.collection.as_deref() {
            Some(name) => normalize_collection_name(name)
                .map_err(|reason| McpError::invalid_params(reason, None))?,
            None => match args.repo_id.as_deref() {
                Some(repo_id) => normalize_collection_name(repo_id)
                    .map_err(|reason| McpError::invalid_params(reason, None))?,
                None => {
                    return Err(McpError::invalid_params(
                        "collection parameter is required (no repo_id available for auto-resolution)",
                        None,
                    ));
                }
            },
        };
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
                let collection_id = match args.collection.as_deref() {
                    Some(name) => normalize_collection_name(name)
                        .map_err(|reason| McpError::invalid_params(reason, None))?,
                    None => match args.repo_id.as_deref() {
                        Some(repo_id) => normalize_collection_name(repo_id)
                            .map_err(|reason| McpError::invalid_params(reason, None))?,
                        None => {
                            return Err(McpError::invalid_params(
                                "collection parameter is required (no repo_id available for auto-resolution)",
                                None,
                            ));
                        }
                    },
                };
                let collection_str = collection_id.to_string();
                match self.indexing_service.clear_collection(&collection_id).await {
                    Ok(()) => Ok(ResponseFormatter::format_clear_index(&collection_str)),
                    Err(e) => Ok(ResponseFormatter::format_indexing_error(
                        &format!("Failed to clear collection {collection_str}: {e}"),
                        &error_path,
                    )),
                }
            }
        }
    }
}
