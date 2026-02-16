use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::utils::source::{DeclScanConfig, count_matches_in_block, scan_decl_blocks};
use crate::validators::solid::violation::SolidViolation;

/// Validates declarations by counting members matched by regex patterns.
///
/// # Errors
/// Returns an error when required patterns cannot be loaded or scanning fails.
pub fn validate_decl_member_count<F>(
    config: &ValidationConfig,
    decl_pattern_key: &str,
    member_pattern_key: &str,
    max_allowed: usize,
    make_violation: F,
) -> Result<Vec<SolidViolation>>
where
    F: Fn(std::path::PathBuf, usize, &str, usize, usize) -> SolidViolation,
{
    let decl_pattern = required_pattern(decl_pattern_key)?;
    let member_pattern = required_pattern(member_pattern_key)?;
    scan_decl_blocks(
        config,
        &DeclScanConfig {
            decl_pattern,
            member_fn_pattern: member_pattern,
            count_fn: count_matches_in_block,
            max_allowed,
        },
        make_violation,
    )
}

#[derive(Clone, Copy)]
/// Kind of declaration evaluated by member-count checks.
pub enum MemberCountKind {
    /// Trait declaration.
    Trait,
    /// Impl declaration.
    Impl,
}

/// Input bundle used to build a member-count violation.
pub struct MemberCountInput<'a> {
    /// Source file path.
    pub file: std::path::PathBuf,
    /// 1-based line number.
    pub line: usize,
    /// Item name (trait or type).
    pub item_name: &'a str,
    /// Number of matched methods.
    pub method_count: usize,
    /// Configured limit for methods.
    pub max_allowed: usize,
}

/// Constructs a concrete SOLID violation for member-count checks.
#[must_use]
pub fn make_member_count_violation(
    kind: MemberCountKind,
    input: MemberCountInput<'_>,
) -> SolidViolation {
    let MemberCountInput {
        file,
        line,
        item_name,
        method_count,
        max_allowed,
    } = input;

    match kind {
        MemberCountKind::Trait => SolidViolation::TraitTooLarge {
            file,
            line,
            trait_name: item_name.to_owned(),
            method_count,
            max_allowed,
            suggestion: "Consider splitting into smaller, focused traits".to_owned(),
            severity: Severity::Warning,
        },
        MemberCountKind::Impl => SolidViolation::ImplTooManyMethods {
            file,
            line,
            type_name: item_name.to_owned(),
            method_count,
            max_allowed,
            suggestion:
                "Consider splitting into smaller, focused impl blocks or extracting to traits"
                    .to_owned(),
            severity: Severity::Warning,
        },
    }
}
