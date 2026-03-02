//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Project detection with auto-registration via linkme.

mod cargo;
mod common;
mod detector;
mod go;
mod maven;
mod npm;
mod python;

pub use cargo::CargoDetector;
pub use detector::detect_all_projects;
pub use go::GoDetector;
pub use maven::MavenDetector;
pub use npm::NpmDetector;
pub use python::PythonDetector;

// ---------------------------------------------------------------------------
// Linkme Registration (moved from mcb-infrastructure for CA compliance)
// ---------------------------------------------------------------------------

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::ProjectDetectorService;
use mcb_domain::registry::project_detection::{
    PROJECT_DETECTION_SERVICES, ProjectDetectionServiceEntry,
};

/// Thin adapter implementing the domain port via the detect-all facade.
struct UniversalProjectDetector {
    detect_fn: Arc<
        dyn for<'a> Fn(&'a Path) -> Pin<Box<dyn Future<Output = Vec<ProjectType>> + Send + 'a>>
            + Send
            + Sync,
    >,
}

#[async_trait]
impl ProjectDetectorService for UniversalProjectDetector {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType> {
        (self.detect_fn)(path).await
    }
}

#[linkme::distributed_slice(PROJECT_DETECTION_SERVICES)]
static UNIVERSAL_PROJECT_DETECTION_ENTRY: ProjectDetectionServiceEntry =
    ProjectDetectionServiceEntry {
        name: "universal",
        description: "Universal project detector using all language-specific detectors",
        build: |_config| {
            let detect_fn: Arc<
                dyn for<'a> Fn(
                        &'a Path,
                    )
                        -> Pin<Box<dyn Future<Output = Vec<ProjectType>> + Send + 'a>>
                    + Send
                    + Sync,
            > = Arc::new(|path| Box::pin(detect_all_projects(path)));
            Ok(Arc::new(UniversalProjectDetector { detect_fn }))
        },
    };
