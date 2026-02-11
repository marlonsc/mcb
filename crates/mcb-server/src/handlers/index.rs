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

    /// Validates indexing request arguments and extracts the target path and normalized collection id.
    ///
    /// Checks that `args.path` is present and points to an existing directory, and derives the collection
    /// identifier from `args.collection` (defaults to `"default"`). Returns a formatted `CallToolResult`
    /// describing the validation error when the path is missing, does not exist, or is not a directory.
    ///
    /// # Parameters
    ///
    /// - `args`: Indexing arguments containing `path` and optional `collection`.
    ///
    /// # Returns
    ///
    /// A `(PathBuf, CollectionId)` tuple with the validated directory path and the normalized collection id,
    /// or a `CallToolResult` containing a user-facing error message.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct IndexArgs with a valid directory and optional collection, then validate:
    /// // let args = IndexArgs { path: Some("/some/dir".into()), collection: Some("my_col".into()), ... };
    /// // let (path, collection_id) = validate_request(&args).expect("valid request");
    /// ```
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
        let collection_id = normalize_collection_name(collection_name);
        Ok((path, collection_id))
    }

    /// Dispatches an indexing-related command from parsed arguments and returns a formatted tool response.
    ///
    /// This method validates the provided `IndexArgs` and performs one of three actions:
    /// - Start: validate request inputs, invoke the indexing service to index a codebase, and return a formatted success or error response.
    /// - Status: query the indexing service for current status and return a formatted status response.
    /// - Clear: normalize the target collection name, attempt to clear that collection via the indexing service, and return a formatted confirmation or error response.
    ///
    /// # Returns
    ///
    /// `Ok(CallToolResult)` containing a formatted response appropriate to the requested action; `Err(McpError)` only when the supplied arguments fail validation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Construct handler and args (setup omitted)
    /// let handler = /* IndexHandler::new(indexing_service) */;
    /// let params = Parameters(IndexArgs { action: IndexAction::Status, collection: None, path: None });
    /// let result = handler.handle(params).await;
    /// match result {
    ///     Ok(call_tool_result) => println!("{:?}", call_tool_result),
    ///     Err(e) => eprintln!("Invalid parameters: {:?}", e),
    /// }
    /// ```
    #[tracing::instrument(skip_all)]
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
                let milvus_collection = normalize_collection_name(collection_name);
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