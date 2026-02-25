//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Validation macros — used by all validators in this module.
//!
//! - `validators.rs`: Registry and trait macros (`mk_validators!`, `impl_validator!`)
//! - `violations.rs`: Violation enum generator (`define_violations!`)
//! - `naming.rs`: Naming/CA helper (`apply_ca_rule!`)

pub mod naming;
pub mod validators;
pub mod violations;
