//! Implementation quality validation module
//!
//! **Documentation**: [`docs/modules/validate.md#implementation`](../../../../../docs/modules/validate.md#implementation)

mod validator;
mod violation;

pub use self::validator::ImplementationQualityValidator;
pub use self::violation::ImplementationViolation;
pub use crate::constants::implementation::{HARDCODED_RETURN_PATTERNS, STUB_SKIP_FILE_KEYWORDS};

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "implementation",
        description: "Validates implementation quality patterns (empty methods, hardcoded returns, stubs)",
        build: |root| {
            Ok(Box::new(ImplementationQualityValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
