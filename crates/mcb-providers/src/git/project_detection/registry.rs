//! Project detector registry (linkme distributed slice).

use mcb_domain::ports::providers::project_detection::ProjectDetectorEntry;

/// Distributed slice for auto-registration of project detectors
#[allow(unsafe_code)]
#[linkme::distributed_slice]
#[allow(unsafe_code)]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];
