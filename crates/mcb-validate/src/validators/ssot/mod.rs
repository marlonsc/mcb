/// SSOT validator implementation.
pub mod validator;
/// SSOT violation types.
pub mod violation;

pub use self::validator::SsotValidator;
pub use self::violation::SsotViolation;
