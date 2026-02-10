//! SOLID principles validation module

mod validator;
mod violation;

pub use validator::SolidValidator;
pub use violation::SolidViolation;
