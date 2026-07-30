//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Project Service Implementation
//!
//! # Overview
//! The `ProjectService` provides capabilities for identifying and classifying software projects
//! within a directory structure. It acts as the infrastructure adapter for project detection logic.
//!
//! # Responsibilities
//! - **Language Detection**: Identifying primary programming languages.
//! - **Framework Recognition**: Detecting project frameworks (e.g., React, Django, cargo).
//! - **Monorepo Handling**: Scanning nested projects within a workspace.

use std::fmt;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::ProjectDetectorService;

/// Type alias for the async detection function injected by the DI composition root.
///
/// Accepts a path and returns detected project types. The concrete implementation
/// is provided by `mcb-providers` via the DI layer (CA-compliant).
pub type DetectAllFn = Arc<
    dyn for<'a> Fn(&'a Path) -> Pin<Box<dyn Future<Output = Vec<ProjectType>> + Send + 'a>>
        + Send
        + Sync,
>;

/// Infrastructure service for project detection and scanning.
///
/// Wraps an injected detection function to recursively discover
/// and classify projects across the workspace.
pub struct ProjectService {
    detect_fn: DetectAllFn,
}

impl fmt::Debug for ProjectService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectService").finish()
    }
}

impl ProjectService {
    /// Creates a project detector service instance with the given detection function.
    #[must_use]
    pub fn new(detect_fn: DetectAllFn) -> Self {
        Self { detect_fn }
    }
}

#[async_trait]
impl ProjectDetectorService for ProjectService {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType> {
        (self.detect_fn)(path).await
    }
}

// NOTE: Linkme registration for the universal project detection service has been
// moved to `mcb-providers::project_detection` to respect Clean Architecture
// boundaries (infra must not depend on providers).
