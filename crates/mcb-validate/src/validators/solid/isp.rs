use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::validators::solid::utils::{count_matches_in_block, scan_decl_blocks};
use crate::validators::solid::violation::SolidViolation;

/// ISP: Check for traits with too many methods
///
/// # Errors
/// Returns an error if pattern compilation fails.
pub fn validate_isp(
    config: &ValidationConfig,
    max_trait_methods: usize,
) -> Result<Vec<SolidViolation>> {
    let trait_pattern = required_pattern("SOLID001.trait_decl")?;
    let fn_pattern = required_pattern("SOLID001.fn_decl")?;

    scan_decl_blocks(
        config,
        trait_pattern,
        fn_pattern,
        count_matches_in_block,
        max_trait_methods,
        |file, line, trait_name, method_count, max_allowed| SolidViolation::TraitTooLarge {
            file,
            line,
            trait_name: trait_name.to_string(),
            method_count,
            max_allowed,
            suggestion: "Consider splitting into smaller, focused traits".to_string(),
            severity: Severity::Warning,
        },
    )
}
