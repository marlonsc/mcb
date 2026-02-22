//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Browse handlers for Admin UI
//!
//! REST API handlers for browsing indexed collections, files, and code chunks.
//! Provides navigation capabilities for the Admin UI code browser.
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! | ------ | -------- | ------------- |
//! | `/collections` | GET | List all indexed collections |
//! | `/collections/:name/files` | GET | List files in a collection |
//! | `/collections/:name/files/*path/chunks` | GET | Get chunks for a file |

use std::sync::Arc;

use crate::constants::limits::DEFAULT_BROWSE_FILES_LIMIT;
use axum::Json as AxumJson;
use axum::extract::{Path, Query, State as AxumState};
use mcb_domain::info;
use mcb_domain::ports::HighlightServiceInterface;
use mcb_domain::ports::VectorStoreBrowser;
use mcb_domain::value_objects::CollectionId;
use mcb_domain::value_objects::FileTreeNode;

use super::models::{
    ChunkDetailResponse, ChunkListResponse, CollectionInfoResponse, CollectionListResponse,
    FileInfoResponse, FileListResponse,
};
use crate::admin::auth::AxumAdminAuth;
use crate::admin::error::{AdminError, AdminResult};
use crate::constants::LIST_FILE_PATHS_LIMIT;

/// Query parameters for browse collection file listing (Axum).
#[derive(Debug, serde::Deserialize)]
pub struct BrowseFilesQuery {
    /// Maximum number of files to return (default: 100).
    pub limit: Option<usize>,
}

/// Browse handler state containing the vector store browser
#[derive(Clone)]
pub struct BrowseState {
    /// Vector store browser for collection/file navigation
    pub browser: Arc<dyn VectorStoreBrowser>,
    /// Highlight service for code syntax highlighting
    pub highlight_service: Arc<dyn HighlightServiceInterface>,
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
    #[must_use]
    pub fn not_found(resource: &str) -> Self {
        Self::new(format!("{resource} not found"), "NOT_FOUND")
    }

    /// Creates an internal error response
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(message, "INTERNAL_ERROR")
    }
}

/// Axum handler: list all indexed collections.
///
/// # Errors
/// Returns `500` for backend failures.
pub async fn list_collections_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<BrowseState>>,
) -> AdminResult<CollectionListResponse> {
    info!("browse", "list_collections called");
    let collections = state
        .browser
        .list_collections()
        .await
        .map_err(|e| AdminError::internal(e.to_string()))?;

    let collection_responses = collections
        .into_iter()
        .map(|c| CollectionInfoResponse {
            id: c.id.to_string(),
            name: c.name,
            vector_count: c.vector_count,
            file_count: c.file_count,
            last_indexed: c.last_indexed,
            provider: c.provider,
        })
        .collect::<Vec<_>>();

    let total = collection_responses.len();
    Ok(AxumJson(CollectionListResponse {
        collections: collection_responses,
        total,
    }))
}

/// Axum handler: list files in a collection.
///
/// # Errors
/// Returns `404` for unknown collections and `500` for backend failures.
pub async fn list_collection_files_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<BrowseState>>,
    Path(name): Path<String>,
    Query(params): Query<BrowseFilesQuery>,
) -> AdminResult<FileListResponse> {
    info!("browse", "list_collection_files called");
    let limit = params.limit.unwrap_or(DEFAULT_BROWSE_FILES_LIMIT);
    let collection = CollectionId::from_string(&name);

    let files = state
        .browser
        .list_file_paths(&collection, limit)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                AdminError::not_found("Collection not found")
            } else {
                AdminError::internal(error_msg)
            }
        })?;

    let file_responses = files
        .into_iter()
        .map(|f| FileInfoResponse {
            path: f.path,
            chunk_count: f.chunk_count,
            language: f.language,
            size_bytes: f.size_bytes,
        })
        .collect::<Vec<_>>();

    let total = file_responses.len();
    Ok(AxumJson(FileListResponse {
        files: file_responses,
        total,
        collection: name,
    }))
}

/// Returns chunk list for a file in a collection (Axum).
///
/// # Errors
/// Returns `404` when file or collection is not found, `500` on browser or highlight errors.
pub async fn get_file_chunks_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<BrowseState>>,
    Path((name, path)): Path<(String, String)>,
) -> AdminResult<ChunkListResponse> {
    info!("browse", "get_file_chunks called");
    let file_path = path.replace('\\', "/");
    let collection_id = CollectionId::from_string(&name);

    let chunks = state
        .browser
        .get_chunks_by_file(&collection_id, &file_path)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                AdminError::not_found("File or collection")
            } else {
                AdminError::internal(error_msg)
            }
        })?;

    let mut chunk_responses = Vec::with_capacity(chunks.len());
    for c in chunks {
        let line_count = c.content.lines().count() as u32;
        let end_line = c.start_line.saturating_add(line_count.saturating_sub(1));

        let (highlighted_html, content, language) = match state
            .highlight_service
            .highlight(&c.content, &c.language)
            .await
        {
            Ok(h) => {
                let html =
                    mcb_infrastructure::services::highlight_renderer::HtmlRenderer::render(&h);
                (html, c.content, c.language)
            }
            Err(_) => {
                let fallback = mcb_domain::value_objects::browse::HighlightedCode::new(
                    c.content,
                    vec![],
                    c.language,
                );
                let html = mcb_infrastructure::services::highlight_renderer::HtmlRenderer::render(
                    &fallback,
                );
                (html, fallback.original, fallback.language)
            }
        };

        chunk_responses.push(ChunkDetailResponse {
            id: c.id,
            content,
            highlighted_html,
            file_path: c.file_path,
            start_line: c.start_line,
            end_line,
            language,
            score: c.score,
        });
    }

    let total = chunk_responses.len();
    Ok(AxumJson(ChunkListResponse {
        chunks: chunk_responses,
        file_path,
        collection: name,
        total,
    }))
}

/// Returns file tree for a collection (Axum).
///
/// # Errors
/// Returns `404` when collection is not found, `500` on browser errors.
pub async fn get_collection_tree_axum(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<BrowseState>>,
    Path(name): Path<String>,
) -> AdminResult<FileTreeNode> {
    info!("browse", "get_collection_tree called");
    let collection_id = CollectionId::from_string(&name);
    let files = state
        .browser
        .list_file_paths(&collection_id, LIST_FILE_PATHS_LIMIT)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("does not exist") {
                AdminError::not_found("Collection")
            } else {
                AdminError::internal(error_msg)
            }
        })?;

    let mut root = FileTreeNode::directory(&name, "");
    for file in files {
        let parts = file.path.split('/').collect::<Vec<_>>();
        insert_into_tree(&mut root, &parts, &file);
    }

    root.sort_children();
    Ok(AxumJson(root))
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
                name.to_owned()
            } else {
                format!("{}/{}", node.path, name)
            };
            let mut dir_node = FileTreeNode::directory(name, &current_path);
            insert_into_tree(&mut dir_node, &parts[1..], file);
            *node = node.clone().with_child(dir_node);
        }
    }
}
