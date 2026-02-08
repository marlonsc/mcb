//! Clean Architecture validation module

mod validator;
mod violation;

pub use validator::CleanArchitectureValidator;
pub use violation::CleanArchitectureViolation;
