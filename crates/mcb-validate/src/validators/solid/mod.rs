//! SOLID principles validation module
//!
//! **Documentation**: [`docs/modules/validate.md#solid`](../../../../../docs/modules/validate.md#solid)

mod isp;
mod lsp;
mod ocp;
mod srp;
mod utils;
mod validator;
mod violation;

pub use self::utils::{
    MemberCountInput, MemberCountKind, make_member_count_violation, validate_decl_member_count,
};
pub use self::validator::SolidValidator;
pub use self::violation::SolidViolation;
