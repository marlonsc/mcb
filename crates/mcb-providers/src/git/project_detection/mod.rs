//! Project detection with auto-registration via linkme.
#![allow(unsafe_code)]

mod cargo;
mod detector;
mod go;
mod maven;
mod npm;
mod python;
mod registry;

pub use cargo::CargoDetector;
pub use detector::detect_all_projects;
pub use go::GoDetector;
pub use maven::MavenDetector;
pub use npm::NpmDetector;
pub use python::PythonDetector;
pub use registry::PROJECT_DETECTORS;
