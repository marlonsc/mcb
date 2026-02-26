//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Project detection with auto-registration via linkme.
#![allow(unsafe_code)]

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
