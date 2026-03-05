//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

mod helpers;
mod validator;

pub use self::validator::{DocumentationValidator, DocumentationViolation};

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_DOCUMENTATION,
    "Validates documentation standards",
    |root| {
        Ok(Box::new(DocumentationValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
