//! SOLID principles validation module

pub mod constants;
mod isp;
mod lsp;
mod ocp;
mod srp;
mod validator;
mod violation;

pub use self::validator::SolidValidator;
pub use self::violation::SolidViolation;
