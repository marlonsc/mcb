//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Re-exports performance validator constants from the central constants module.

pub use crate::constants::performance::{
    ARC_MUTEX_OVERUSE_PATTERNS, CLONE_REGEX, CONTEXT_TRUNCATION_LENGTH,
    INEFFICIENT_ITERATOR_PATTERNS, INEFFICIENT_STRING_PATTERNS, LOOP_ALLOCATION_PATTERNS,
};
