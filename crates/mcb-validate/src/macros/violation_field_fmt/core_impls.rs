//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use super::impl_violation_field_fmt;

impl_violation_field_fmt!(
    PathBuf => |value: &PathBuf| value.display().to_string()
);

impl_violation_field_fmt!(String, &str, usize, u32, i32, i64);
