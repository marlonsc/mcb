//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Re-exports SOLID validator constants from the central constants module.

pub use crate::constants::solid::{
    MAX_AFFIX_LENGTH, MAX_UNRELATED_STRUCTS_PER_FILE, MIN_AFFIX_LENGTH,
    MIN_NAMES_FOR_RELATION_CHECK, MIN_STRING_MATCH_ARMS_FOR_DISPATCH,
    MIN_WORD_LENGTH_FOR_COMPARISON,
};
