//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Performance Pattern Validation
//!
//! This module provides the `PerformanceValidator` which identifies common performance
//! anti-patterns in Rust code. It focuses on identifying clone abuse, unnecessary
//! allocations in loops, and suboptimal synchronization patterns.
//!
//!
//! Detects performance anti-patterns that PMAT and Clippy might miss:
//! - Clone abuse (redundant clones, clones in loops)
//! - Allocation patterns (Vec/String in loops)
//! - Arc/Mutex overuse
//! - Inefficient iterator patterns

mod loop_checks;
mod loops;
mod pattern_checks;
mod validator;
mod violation;

pub use self::validator::PerformanceValidator;
pub use self::violation::PerformanceViolation;

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "performance",
        description: "Validates performance patterns (clones, allocations, Arc/Mutex usage)",
        build: |root| {
            Ok(Box::new(PerformanceValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
