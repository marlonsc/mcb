//! SOLID principles validation module

use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::utils::source::{count_matches_in_block, scan_decl_blocks};

pub mod constants;
mod isp;
mod lsp;
mod ocp;
mod srp;
mod validator;
mod violation;

pub use self::validator::SolidValidator;
pub use self::violation::SolidViolation;

fn validate_decl_member_count<F>(
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
        decl_pattern,
        member_pattern,
        count_matches_in_block,
        max_allowed,
        make_violation,
    )
}

#[derive(Clone, Copy)]
pub(super) enum MemberCountKind {
    Trait,
    Impl,
}

pub(super) struct MemberCountInput<'a> {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub item_name: &'a str,
    pub method_count: usize,
    pub max_allowed: usize,
}

pub(super) fn make_member_count_violation(
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
