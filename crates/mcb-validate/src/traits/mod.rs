#[path = "../validators/traits/validator.rs"]
pub mod validator;
#[path = "../validators/traits/violation.rs"]
pub mod violation;

pub use self::validator::{Validator, ValidatorRegistry};
pub use self::violation::{Violation, ViolationCategory};
