/// Domain layer purity checks.
pub mod domain_purity;
/// Duplicate string detection checks.
pub mod duplicate_strings;
/// File placement checks.
pub mod file_placement;
/// Cross-layer dependency violation checks.
pub mod layer_violations;
/// Magic number checks.
pub mod magic_numbers;
/// Strict directory rule checks.
pub mod strict_directory;
/// Trait placement checks.
pub mod trait_placement;
/// Organization validator orchestrator.
pub mod validator;
/// Organization violation model.
pub mod violation;

pub use self::validator::OrganizationValidator;
pub use self::violation::OrganizationViolation;
