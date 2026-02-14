//! Project detector registry (linkme distributed slice).
// linkme distributed_slice uses #[link_section] internally
#![allow(unsafe_code)]

use mcb_domain::ports::providers::project_detection::ProjectDetectorEntry;

/// Distributed slice for auto-registration of project detectors
#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];
