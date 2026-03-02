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

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "organization",
        description: "Validates code organization patterns",
        build: |root| {
            Ok(Box::new(OrganizationValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
