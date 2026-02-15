//! SOLID principles validation module

mod isp;
mod lsp;
mod ocp;
mod srp;
/// Utility helpers for SOLID validation analysis.
/// Validation utilities
pub mod utils;
mod validator;
mod violation;

pub use self::validator::SolidValidator;
pub use self::violation::SolidViolation;
