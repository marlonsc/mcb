use super::MemberCountInput;
use super::MemberCountKind;
use super::make_member_count_violation;
use super::validate_decl_member_count;
use crate::Result;
use crate::ValidationConfig;
use crate::validators::solid::violation::SolidViolation;

/// ISP: Check for traits with too many methods
///
/// # Errors
/// Returns an error if pattern compilation fails.
pub fn validate_isp(
    config: &ValidationConfig,
    max_trait_methods: usize,
) -> Result<Vec<SolidViolation>> {
    validate_decl_member_count(
        config,
        "SOLID001.trait_decl",
        "SOLID001.fn_decl",
        max_trait_methods,
        |file, line, trait_name, method_count, max_allowed| {
            make_member_count_violation(
                MemberCountKind::Trait,
                MemberCountInput {
                    file,
                    line,
                    item_name: trait_name,
                    method_count,
                    max_allowed,
                },
            )
        },
    )
}
