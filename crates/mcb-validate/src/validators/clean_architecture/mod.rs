//! Clean Architecture validation module

mod validator;
mod violation;

pub use self::validator::CleanArchitectureValidator;
pub use self::violation::CleanArchitectureViolation;
