//! Clean Architecture validation module

mod violation;
mod validator;

pub use violation::CleanArchitectureViolation;
pub use validator::CleanArchitectureValidator;
