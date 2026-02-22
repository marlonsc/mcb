//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#service-ports)
//!
//! Provides project domain definitions.
use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::ProjectType;

#[async_trait]
/// Defines behavior for `ProjectDetectorService`.
pub trait ProjectDetectorService: Send + Sync {
    /// Performs the detect all operation.
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}
