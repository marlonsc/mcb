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

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_SOLID,
    "Validates SOLID principles",
    |root| {
        Ok(Box::new(SolidValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
