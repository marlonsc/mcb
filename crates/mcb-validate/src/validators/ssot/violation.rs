use std::path::PathBuf;

use crate::Severity;
use crate::define_violations;
use crate::traits::violation::ViolationCategory;

define_violations! {
    ViolationCategory::Organization,
    pub enum SsotViolation {
        #[doc = "Duplicate declaration of the same public type or trait."]
        #[violation(
            id = "SSOT001",
            severity = Error,
            message = "Duplicate port declaration '{declaration_name}' found across files: {locations}",
            suggestion = "Keep a single declaration under the canonical ports tree and remove duplicates"
        )]
        DuplicatePortDeclaration {
            file: PathBuf,
            line: usize,
            declaration_name: String,
            locations: String,
            severity: Severity,
        },
        #[doc = "Forbidden import of the legacy repositories compatibility path."]
        #[violation(
            id = "SSOT002",
            severity = Error,
            message = "Forbidden legacy import: {file}:{line} uses '{import_path}'",
            suggestion = "Replace with mcb_domain::ports::{TypeName} imports"
        )]
        ForbiddenLegacyImport {
            file: PathBuf,
            line: usize,
            import_path: String,
            severity: Severity,
        },
        #[doc = "Forbidden legacy schema symbol usage."]
        #[violation(
            id = "SSOT003",
            severity = Error,
            message = "Forbidden legacy schema symbol: {file}:{line} uses '{symbol_name}'",
            suggestion = "Remove legacy schema symbols and use the canonical schema SSOT APIs"
        )]
        ForbiddenLegacySchemaSymbol {
            file: PathBuf,
            line: usize,
            symbol_name: String,
            severity: Severity,
        },
        #[doc = "Forbidden macro path referencing `schema::memory` internals."]
        #[violation(
            id = "SSOT004",
            severity = Error,
            message = "Forbidden schema macro path: {file}:{line} uses '{macro_path}'",
            suggestion = "Stop referencing $crate::schema::memory::* in macros"
        )]
        ForbiddenSchemaMemoryMacroPath {
            file: PathBuf,
            line: usize,
            macro_path: String,
            severity: Severity,
        },
        #[doc = "Forbidden legacy schema import path."]
        #[violation(
            id = "SSOT005",
            severity = Error,
            message = "Forbidden legacy schema import: {file}:{line} uses '{import_path}'",
            suggestion = "Remove imports of MemorySchema/ProjectSchema and migrate to canonical schema modules"
        )]
        ForbiddenLegacySchemaImport {
            file: PathBuf,
            line: usize,
            import_path: String,
            severity: Severity,
        },
        #[doc = "Forbidden root-level schema path usage."]
        #[violation(
            id = "SSOT006",
            severity = Error,
            message = "Forbidden root schema path: {file}:{line} uses '{path}'",
            suggestion = "Use canonical mcb_domain::schema::* paths instead of root-level schema symbols"
        )]
        ForbiddenRootSchemaPath {
            file: PathBuf,
            line: usize,
            path: String,
            severity: Severity,
        },
    }
}
