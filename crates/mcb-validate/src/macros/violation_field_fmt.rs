//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
mod collection_impls;
mod core_impls;
mod domain_impls;
mod scalar_impls;

macro_rules! impl_violation_field_fmt {
    ($($type:ty),+ $(,)?) => {
        $(
            impl $crate::macros::violation_field_fmt::ViolationFieldFmt for $type {
                fn fmt_field(&self) -> String {
                    self.to_string()
                }
            }
        )+
    };
    ($( $type:ty => $body:expr ),+ $(,)?) => {
        $(
            impl $crate::macros::violation_field_fmt::ViolationFieldFmt for $type {
                fn fmt_field(&self) -> String {
                    ($body)(self)
                }
            }
        )+
    };
}

pub(crate) use impl_violation_field_fmt;

/// Formats violation fields into stable template strings.
///
/// # Example
///
/// ```rust
/// use mcb_validate::macros::violation_field_fmt::ViolationFieldFmt;
/// assert_eq!(3usize.fmt_field(), "3");
/// ```
pub trait ViolationFieldFmt {
    /// Converts a field into the string form used by message templates.
    fn fmt_field(&self) -> String;
}
