//! Module and directory organization validation
//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
pub mod domain_purity;
pub mod duplicate_strings;
pub mod file_placement;
pub mod layer_violations;
pub mod magic_numbers;
pub mod strict_directory;
pub mod trait_placement;
pub mod validator;
pub mod violation;

pub use self::validator::OrganizationValidator;
pub use self::violation::OrganizationViolation;

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_ORGANIZATION,
    "Validates code organization patterns",
    |root| {
        Ok(Box::new(OrganizationValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
