//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::violation::PatternViolation;

pub fn check_result_types(_path: &Path, _content: &str) -> crate::Result<Vec<PatternViolation>> {
    Ok(Vec::new())
}
