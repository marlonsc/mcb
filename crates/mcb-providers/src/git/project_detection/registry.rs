//! Project detector registry (linkme distributed slice).

use mcb_domain::ports::providers::project_detection::ProjectDetectorEntry;

/// Distributed slice for auto-registration of project detectors
#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];
