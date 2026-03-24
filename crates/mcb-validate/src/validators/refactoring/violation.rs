//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#refactoring)
//!
use std::path::PathBuf;

use crate::Severity;
use crate::define_violations;
use mcb_domain::ports::validation::ViolationCategory;

define_violations! {
    dynamic_severity,
    ViolationCategory::Refactoring,
    pub enum RefactoringViolation {
        /// Import statement referencing non-existent module/item
        #[violation(
            id = "REF001",
            severity = Warning,
            message = "Orphan import at {file}:{line} - '{import_path}' - {suggestion}",
            suggestion = "{suggestion}"
        )]
        OrphanImport {
            file: PathBuf,
            line: usize,
            import_path: String,
            suggestion: String,
            severity: Severity,
        },
        /// Same type name defined in multiple locations (incomplete migration)
        #[violation(
            id = "REF002",
            severity = Warning,
            message = "Duplicate definition '{type_name}' in [{locations}] - {suggestion}",
            suggestion = "{suggestion}"
        )]
        DuplicateDefinition {
            type_name: String,
            locations: Vec<PathBuf>,
            suggestion: String,
            severity: Severity,
        },
        /// New source file without corresponding test file
        #[violation(
            id = "REF003",
            severity = Warning,
            message = "Missing test file for {source_file} - expected {expected_test}",
            suggestion = "Create test file {expected_test} for source {source_file}"
        )]
        MissingTestFile {
            source_file: PathBuf,
            expected_test: PathBuf,
            severity: Severity,
        },
        /// pub use/mod statement for item that doesn't exist
        #[violation(
            id = "REF004",
            severity = Warning,
            message = "Stale re-export at {file}:{line} - '{re_export}'",
            suggestion = "Remove or update the re-export '{re_export}'"
        )]
        StaleReExport {
            file: PathBuf,
            line: usize,
            re_export: String,
            severity: Severity,
        },
        /// File/module that was deleted but is still referenced
        #[violation(
            id = "REF005",
            severity = Warning,
            message = "Reference to deleted module at {referencing_file}:{line} - '{deleted_module}'",
            suggestion = "Remove the mod declaration for '{deleted_module}' or create the module file"
        )]
        DeletedModuleReference {
            referencing_file: PathBuf,
            line: usize,
            deleted_module: String,
            severity: Severity,
        },
        /// Dead code left from refactoring (unused after move)
        #[violation(
            id = "REF006",
            severity = Warning,
            message = "Dead {item_type} '{item_name}' from refactoring at {file}",
            suggestion = "Remove the unused {item_type} '{item_name}' or use it"
        )]
        RefactoringDeadCode {
            file: PathBuf,
            item_name: String,
            item_type: String,
            severity: Severity,
        },
    }
}
