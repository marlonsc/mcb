//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Async Pattern Validation
//!
//! Detects async-specific anti-patterns based on Tokio documentation:
//! - Blocking in async (`std::thread::sleep`, `std::sync::Mutex` in async)
//! - `block_on()` in async context
//! - Spawn patterns (missing `JoinHandle` handling)
//! - Wrong mutex types in async code

mod block_on;
mod blocking;
pub mod constants;
mod helpers;
mod mutex;
mod spawn;
mod validator;
mod violation;

pub use self::validator::AsyncPatternValidator;
pub use self::violation::AsyncViolation;
pub(crate) use helpers::for_each_async_fn_line;
