//! SOLID principles validation module

mod validator;
mod violation;

pub use self::validator::SolidValidator;
pub use self::violation::SolidViolation;
