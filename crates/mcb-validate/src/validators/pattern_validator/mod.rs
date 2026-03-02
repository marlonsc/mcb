//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Pattern Compliance Validation
//!
//! This module provides the `PatternValidator` which ensures code patterns across the
//! workspace follow established best practices and architectural constraints.
//! It validates Dependency Injection (DI) usage, async trait implementation details,
//! and consistency in Result/Error types.
//!
//! # Code Smells
//!
//! Consider splitting into separate modules for DI, async traits, and result types.
//!
//! Validates code patterns:
//! - DI uses `Arc<dyn Trait>` not `Arc<ConcreteType>`
//! - Async traits have `#[async_trait]` and Send + Sync bounds
//! - Error types use `crate::error::Result<T>`
//! - Provider pattern compliance

mod async_check;
mod di;
mod result_check;
mod validator;
mod violation;

pub use self::validator::PatternValidator;
pub use self::violation::PatternViolation;

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "pattern",
        description: "Validates code patterns (DI, Async, Result types)",
        build: |root| {
            Ok(Box::new(PatternValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
