//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::ViolationFieldFmt;

impl ViolationFieldFmt for crate::Severity {
    fn fmt_field(&self) -> String {
        format!("{self}")
    }
}

impl ViolationFieldFmt for crate::ComponentType {
    fn fmt_field(&self) -> String {
        format!("{self}")
    }
}

impl ViolationFieldFmt for crate::validators::dependency::DependencyCycle {
    fn fmt_field(&self) -> String {
        self.0.join(" -> ")
    }
}
