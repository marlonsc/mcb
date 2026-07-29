//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use super::impl_violation_field_fmt;

impl_violation_field_fmt!(Vec<String> => |value: &Vec<String>| value.join(", "));

impl_violation_field_fmt!(
    Vec<PathBuf> => |value: &Vec<PathBuf>| value
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", "),
    Vec<(PathBuf, usize)> => |value: &Vec<(PathBuf, usize)>| value
        .iter()
        .map(|(p, n)| format!("{}:{}", p.display(), n))
        .collect::<Vec<_>>()
        .join(", "),
    Vec<(PathBuf, usize, String)> => |value: &Vec<(PathBuf, usize, String)>| value
        .iter()
        .map(|(p, n, s)| format!("{}:{}:{}", p.display(), n, s))
        .collect::<Vec<_>>()
        .join(", "),
    Vec<(PathBuf, String)> => |value: &Vec<(PathBuf, String)>| value
        .iter()
        .map(|(p, s)| format!("{}:{}", p.display(), s))
        .collect::<Vec<_>>()
        .join(", ")
);
