//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Highlight Service Use Case
//!
//! # Overview
//! The `HighlightService` provides backend-agnostic syntax highlighting capabilities using
//! Tree-sitter. It parses source code into an abstract syntax tree (AST) to identify
//! tokens and apply semantic highlighting rules, independent of the final output format.
//!
//! # Responsibilities
//! - **Multi-Language Support**: Parsing and highlighting code for supported languages (Rust, Python, JS, etc.).
//! - **Tree-Sitter Integration**: Leveraging widely-used grammars for accurate syntax analysis.
//! - **Abstract Representation**: Producing a generic `HighlightedCode` structure (spans + categories)
//!   that can be rendered to HTML, ANSI, or other formats.

use std::sync::Arc;

use mcb_domain::ports::{HighlightError, HighlightServiceInterface};
use mcb_domain::registry::services::ServiceBuilder;
use mcb_domain::value_objects::browse::HighlightedCode;

use crate::services::highlight_sync_service::{HighlightSyncPort, HighlightSyncService};

/// Concrete highlight service implementation using tree-sitter.
///
/// Delegates all CPU-bound work to [`HighlightSyncService`] running inside
/// `spawn_blocking` so the async executor is never blocked.
pub struct HighlightServiceImpl {
    inner: Arc<dyn HighlightSyncPort>,
}

impl HighlightServiceImpl {
    /// Creates a syntax highlight service wrapping the given sync port.
    #[must_use]
    pub fn new(inner: Arc<dyn HighlightSyncPort>) -> Self {
        Self { inner }
    }
}

impl Default for HighlightServiceImpl {
    fn default() -> Self {
        Self::new(Arc::new(HighlightSyncService::new()))
    }
}

#[async_trait::async_trait]
impl HighlightServiceInterface for HighlightServiceImpl {
    async fn highlight(&self, code: &str, language: &str) -> mcb_domain::Result<HighlightedCode> {
        let code = code.to_owned();
        let language = language.to_owned();
        let inner = Arc::clone(&self.inner);

        let result = tokio::task::spawn_blocking(move || inner.highlight(&code, &language))
            .await
            .map_err(|e| {
                HighlightError::HighlightingFailed(format!("Blocking task failed: {e}"))
            })?;

        result.map_err(mcb_domain::Error::from)
    }
}

mcb_domain::register_service!(
    mcb_utils::constants::SERVICE_NAME_HIGHLIGHT,
    ServiceBuilder::Highlight(|_context| {
        Ok(std::sync::Arc::new(HighlightServiceImpl::new(Arc::new(
            HighlightSyncService::new(),
        ))))
    }),
);
