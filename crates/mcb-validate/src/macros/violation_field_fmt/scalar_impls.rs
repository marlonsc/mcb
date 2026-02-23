//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::ViolationFieldFmt;

impl ViolationFieldFmt for bool {
    fn fmt_field(&self) -> String {
        self.to_string()
    }
}
