//! SOLID principles validation module

mod violation;
mod validator;

pub use violation::SolidViolation;
pub use validator::SolidValidator;
