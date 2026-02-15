mod collection_impls;
mod core_impls;
mod domain_impls;
mod scalar_impls;

/// Formats violation fields into stable template strings.
///
/// # Example
///
/// ```rust
/// use mcb_validate::violation_macro::violation_field_fmt::ViolationFieldFmt;
/// assert_eq!(3usize.fmt_field(), "3");
/// ```
pub trait ViolationFieldFmt {
    /// Converts a field into the string form used by message templates.
    fn fmt_field(&self) -> String;
}
