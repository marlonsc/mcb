#![allow(missing_docs)]
#![allow(unsafe_code)]

use crate::ports::ProjectDetectorEntry;

#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];

#[must_use]
pub fn list_project_detectors() -> Vec<(&'static str, &'static str)> {
    PROJECT_DETECTORS
        .iter()
        .map(|entry| (entry.name, entry.description))
        .collect()
}
