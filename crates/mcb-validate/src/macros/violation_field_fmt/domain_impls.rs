//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::impl_violation_field_fmt;

impl_violation_field_fmt!(crate::Severity, crate::ComponentType);

impl_violation_field_fmt!(
    crate::validators::dependency::DependencyCycle =>
        |value: &crate::validators::dependency::DependencyCycle| value.0.join(" -> ")
);
