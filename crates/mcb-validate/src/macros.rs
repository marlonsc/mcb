//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
//! Violation runtime types (field formatting, file path extraction) and declarative macros.
//!
//! - **Runtime**: `ExtractFilePath`, `violation_field_fmt` — used by `define_violations!` via `$crate::macros::*`.
//! - **Declarative**: `naming` (`apply_ca_rule!`), `validators` (`impl_validator!`, etc.),
//!   `violations` (`define_violations!`) — all exported at crate root via `#[macro_export]`.

mod extract_file_path;
/// Field-formatting traits used by violation message template expansion.
pub mod violation_field_fmt;

/// Naming/CA helper macros.
#[macro_use]
pub mod naming;
/// Validator registry and trait implementation macros.
#[macro_use]
pub mod validators;
/// Violation enum generator macros.
#[macro_use]
pub mod violations;

pub use extract_file_path::ExtractFilePath;
