//! Browse handlers for Admin UI
//!
//! REST API handlers for browsing indexed collections, files, and code chunks.
//! Provides navigation capabilities for the Admin UI code browser.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/collections` | GET | List all indexed collections |
//! | `/collections/:name/files` | GET | List files in a collection |
//! | `/collections/:name/files/*path/chunks` | GET | Get chunks for a file |

use mcb_domain::ports::browse::HighlightService;
use mcb_domain::ports::providers::VectorStoreBrowser;
use mcb_domain::value_objects::CollectionId;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};
use std::sync::Arc;

use super::auth::AdminAuth;
use super::models::{
    ChunkDetailResponse, ChunkListResponse, CollectionInfoResponse, CollectionListResponse,
    FileInfoResponse, FileListResponse,
};
use crate::constants::LIST_FILE_PATHS_LIMIT;
use mcb_domain::value_objects::FileTreeNode;

/// Browse handler state containing the vector store browser
#[derive(Clone)]
pub struct BrowseState {
    /// Vector store browser for collection/file navigation
    pub browser: Arc<dyn VectorStoreBrowser>,
    /// Highlight service for code syntax highlighting
    pub highlight_service: Arc<dyn HighlightService>,
}

/// Error response for browse operations
#[derive(serde::Serialize)]
pub struct BrowseErrorResponse {
    /// Error message
    pub error: String,
    /// Error code for programmatic handling
    pub code: String,
}

impl BrowseErrorResponse {
    fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }

    /// Creates a not found error response
    pub fn not_found(resource: &str) -> Self {
        Self::new(format!("{} not found", resource), "NOT_FOUND")
    }

    /// Creates an internal error response
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(message, "INTERNAL_ERROR")
    }
}

/// List all indexed collections
///
/// Returns a list of all collections with their statistics including
/// vector count, file count, and provider information.
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections")]
pub async fn list_collections(
    _auth: AdminAuth,
    state: &State<BrowseState>,
) -> Result<Json<CollectionListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let collections = state.browser.list_collections().await.map_err(|e| {
        (
            Status::InternalServerError,
            Json(BrowseErrorResponse::internal(e.to_string())),
        )
    })?;

    let collection_responses: Vec<CollectionInfoResponse> = collections
        .into_iter()
        .map(|c| CollectionInfoResponse {
            name: c.id.into_string(),
            vector_count: c.vector_count,
            file_count: c.file_count,
            last_indexed: c.last_indexed,
            provider: c.provider,
        })
        .collect();

    let total = collection_responses.len();
    Ok(Json(CollectionListResponse {
        collections: collection_responses,
        total,
    }))
}

/// List files in a collection
///
/// Returns a list of all indexed files in the specified collection,
/// including chunk counts and language information.
///
/// # Arguments
///
/// * `name` - Collection name
/// * `limit` - Maximum number of files to return (default: 100)
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections/<name>/files?<limit>")]
pub async fn list_collection_files(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    limit: Option<usize>,
) -> Result<Json<FileListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let limit = limit.unwrap_or(100);
    let collection = CollectionId::new(name);

    let files = state
        .browser
        .list_file_paths(&collection, limit)
        .await
        .map_err(|e| {
            // Check if it's a collection not found error
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                (
                    Status::NotFound,
                    Json(BrowseErrorResponse::not_found("Collection")),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(BrowseErrorResponse::internal(error_msg)),
                )
            }
        })?;

    let file_responses: Vec<FileInfoResponse> = files
        .into_iter()
        .map(|f| FileInfoResponse {
            path: f.path,
            chunk_count: f.chunk_count,
            language: f.language,
            size_bytes: f.size_bytes,
        })
        .collect();

    let total = file_responses.len();
    Ok(Json(FileListResponse {
        files: file_responses,
        total,
        collection: name.to_string(),
    }))
}

/// Get code chunks for a specific file
///
/// Returns all code chunks that were extracted from a specific file,
/// ordered by line number. Useful for displaying the full indexed
/// content of a file.
///
/// # Arguments
///
/// * `name` - Collection name
/// * `path` - File path (URL-encoded, can contain slashes)
///
/// # Authentication
///
/// Requires valid admin API key via `X-Admin-Key` header.
#[get("/collections/<name>/chunks/<path..>")]
pub async fn get_file_chunks(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
    path: std::path::PathBuf,
) -> Result<Json<ChunkListResponse>, (Status, Json<BrowseErrorResponse>)> {
    let file_path = path.to_string_lossy().to_string();
    let collection_id = CollectionId::new(name);

    let chunks = state
        .browser
        .get_chunks_by_file(&collection_id, &file_path)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                (
                    Status::NotFound,
                    Json(BrowseErrorResponse::not_found("File or collection")),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(BrowseErrorResponse::internal(error_msg)),
                )
            }
        })?;

    let mut chunk_responses = Vec::with_capacity(chunks.len());
    for c in chunks {
        // Estimate end line from content
        let line_count = c.content.lines().count() as u32;
        let end_line = c.start_line.saturating_add(line_count.saturating_sub(1));

        // Generate server-side highlighting via injected service
        let highlighted = match state
            .highlight_service
            .highlight(&c.content, &c.language)
            .await
        {
            Ok(h) => h,
            Err(_) => mcb_domain::value_objects::browse::HighlightedCode::new(
                c.content.clone(),
                vec![],
                c.language.clone(),
            ),
        };

        let highlighted_html =
            crate::handlers::highlight_service::convert_highlighted_code_to_html(&highlighted);

        chunk_responses.push(ChunkDetailResponse {
            id: c.id,
            content: c.content,
            highlighted_html,
            file_path: c.file_path,
            start_line: c.start_line,
            end_line,
            language: c.language,
            score: c.score,
        });
    }

    let total = chunk_responses.len();
    Ok(Json(ChunkListResponse {
        chunks: chunk_responses,
        file_path,
        collection: name.to_string(),
        total,
    }))
}

/// Get file tree for a collection
///
/// Returns a hierarchical tree structure of all indexed files in the
/// collection, organized by directory. Useful for tree view navigation.
#[get("/collections/<name>/tree")]
pub async fn get_collection_tree(
    _auth: AdminAuth,
    state: &State<BrowseState>,
    name: &str,
) -> Result<Json<FileTreeNode>, (Status, Json<BrowseErrorResponse>)> {
    let collection_id = CollectionId::new(name);
    let files = state
        .browser
        .list_file_paths(&collection_id, LIST_FILE_PATHS_LIMIT)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                (
                    Status::NotFound,
                    Json(BrowseErrorResponse::not_found("Collection")),
                )
            } else {
                (
                    Status::InternalServerError,
                    Json(BrowseErrorResponse::internal(error_msg)),
                )
            }
        })?;

    let mut root = FileTreeNode::directory(name, "");

    for file in files {
        let parts: Vec<&str> = file.path.split('/').collect();
        insert_into_tree(&mut root, &parts, &file);
    }

    root.sort_children();
    Ok(Json(root))
}

fn insert_into_tree(
    node: &mut FileTreeNode,
    parts: &[&str],
    file: &mcb_domain::value_objects::FileInfo,
) {
    if parts.is_empty() {
        return;
    }

    let name = parts[0];
    let is_file = parts.len() == 1;

    if is_file {
        let file_node = FileTreeNode::file(name, &file.path, file.chunk_count, &file.language);
        *node = node.clone().with_child(file_node);
    } else {
        let child_idx = node.children.iter().position(|c| c.name == name);
        if let Some(idx) = child_idx {
            insert_into_tree(&mut node.children[idx], &parts[1..], file);
        } else {
            let current_path = if node.path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", node.path, name)
            };
            let mut dir_node = FileTreeNode::directory(name, &current_path);
            insert_into_tree(&mut dir_node, &parts[1..], file);
            *node = node.clone().with_child(dir_node);
        }
    }
}

// Tests moved to tests/unit/browse_handlers_tests.rs per test organization standards
