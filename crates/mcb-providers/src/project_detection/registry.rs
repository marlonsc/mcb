//! Project detection service registration.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::ProjectType;
use mcb_domain::ports::ProjectDetectorService;

use super::detect_all_projects;

/// An async function that detects project types at a given path.
type DetectFn = dyn for<'a> Fn(&'a Path) -> Pin<Box<dyn Future<Output = Vec<ProjectType>> + Send + 'a>>
    + Send
    + Sync;

/// Thin adapter implementing the domain port via the detect-all facade.
struct UniversalProjectDetector {
    /// The detection function used to discover project types.
    detect_fn: Arc<DetectFn>,
}

#[async_trait]
impl ProjectDetectorService for UniversalProjectDetector {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType> {
        (self.detect_fn)(path).await
    }
}

mcb_domain::register_project_detection_service!(
    mcb_utils::constants::DEFAULT_LANGUAGE_PROVIDER,
    "Universal project detector using all language-specific detectors",
    |_config| {
        let detect_fn: Arc<DetectFn> = Arc::new(|path| Box::pin(detect_all_projects(path)));
        Ok(Arc::new(UniversalProjectDetector { detect_fn }))
    }
);
