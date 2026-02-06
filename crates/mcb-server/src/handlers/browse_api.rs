//! Browse REST API Endpoints
//!
//! REST API for file tree browsing and code highlighting.
//! Part of Phase 8a (Web-first rendering with agnóstico service).

use mcb_domain::ports::browse::BrowseService;
use mcb_domain::value_objects::browse::{FileNode, HighlightedCode};
use rocket::{State, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Request to get file tree
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GetTreeRequest {
    pub root: PathBuf,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_max_depth() -> usize {
    3
}

/// Response with file tree
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetTreeResponse {
    pub success: bool,
    pub tree: Option<FileNode>,
    pub error: Option<String>,
}

/// Request to highlight code
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct HighlightRequest {
    pub code: String,
    pub language: String,
}

/// Response with highlighted code
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct HighlightResponse {
    pub success: bool,
    pub highlighted: Option<HighlightedCode>,
    pub error: Option<String>,
}

/// Get file tree from root path
#[post("/api/browse/tree", format = "json", data = "<req>")]
pub async fn get_tree(
    service: &State<Arc<dyn BrowseService>>,
    req: Json<GetTreeRequest>,
) -> Json<GetTreeResponse> {
    match service.get_file_tree(&req.root, req.max_depth).await {
        Ok(tree) => Json(GetTreeResponse {
            success: true,
            tree: Some(tree),
            error: None,
        }),
        Err(e) => Json(GetTreeResponse {
            success: false,
            tree: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Highlight code with given language
#[post("/api/browse/highlight", format = "json", data = "<req>")]
pub async fn highlight(
    service: &State<Arc<dyn BrowseService>>,
    req: Json<HighlightRequest>,
) -> Json<HighlightResponse> {
    match service.highlight(&req.code, &req.language).await {
        Ok(highlighted) => Json(HighlightResponse {
            success: true,
            highlighted: Some(highlighted),
            error: None,
        }),
        Err(e) => Json(HighlightResponse {
            success: false,
            highlighted: None,
            error: Some(e.to_string()),
        }),
    }
}
