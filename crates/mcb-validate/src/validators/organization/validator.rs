//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
//! Organization validator implementation

use super::{
    domain_purity::validate_domain_traits_only, duplicate_strings::validate_duplicate_strings,
    file_placement::validate_file_placement, layer_violations::validate_layer_violations,
    magic_numbers::validate_magic_numbers, strict_directory::validate_strict_directory,
    trait_placement::validate_trait_placement,
};

crate::create_validator!(
    OrganizationValidator,
    "organization",
    "Validates code organization patterns",
    OrganizationViolation,
    [
        validate_magic_numbers,
        validate_duplicate_strings,
        validate_file_placement,
        validate_trait_placement,
        validate_layer_violations,
        validate_strict_directory,
        validate_domain_traits_only,
    ]
);
