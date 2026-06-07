//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

mod helpers;
mod validator;

pub use self::validator::{DocumentationValidator, DocumentationViolation};

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "documentation",
        description: "Validates documentation standards",
        build: |root| {
            Ok(Box::new(DocumentationValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
