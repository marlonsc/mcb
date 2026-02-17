use std::collections::HashMap;

use mcb_validate::{SsotValidator, SsotViolation, Violation, ViolationCategory};

fn synthetic_files(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(path, content)| ((*path).to_owned(), (*content).to_owned()))
        .collect()
}

#[test]
fn detects_duplicate_port_declarations_from_synthetic_files() {
    let files = synthetic_files(&[
        (
            "ports/admin.rs",
            "pub struct ProviderInfo {\n    pub id: String,\n}\n",
        ),
        (
            "ports/infrastructure/admin.rs",
            "pub struct ProviderInfo {\n    pub id: String,\n}\n",
        ),
    ]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let duplicate_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::DuplicatePortDeclaration { .. }))
        .collect::<Vec<_>>();

    assert_eq!(duplicate_violations.len(), 1);
    match duplicate_violations[0] {
        SsotViolation::DuplicatePortDeclaration {
            declaration_name,
            locations,
            ..
        } => {
            assert_eq!(declaration_name, "ProviderInfo");
            assert!(locations.contains("ports/admin.rs:1"));
            assert!(locations.contains("ports/infrastructure/admin.rs:1"));
        }
        SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenLegacySchemaSymbol { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
    assert_eq!(
        Violation::category(duplicate_violations[0]),
        ViolationCategory::Organization
    );
}

#[test]
fn detects_forbidden_legacy_imports_from_synthetic_files() {
    let files = synthetic_files(&[(
        "application/use_case.rs",
        "use std::sync::Arc;\nuse mcb_domain::repositories::user_repository::UserRepository;\n",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let legacy_import_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::ForbiddenLegacyImport { .. }))
        .collect::<Vec<_>>();

    assert_eq!(legacy_import_violations.len(), 1);
    match legacy_import_violations[0] {
        SsotViolation::ForbiddenLegacyImport {
            file,
            line,
            import_path,
            ..
        } => {
            assert_eq!(file.to_string_lossy(), "application/use_case.rs");
            assert_eq!(*line, 2);
            assert!(import_path.contains("mcb_domain::repositories::"));
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacySchemaSymbol { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}

#[test]
fn reports_both_ssot_violation_types_together() {
    let files = synthetic_files(&[
        (
            "ports/a.rs",
            "pub trait BillingPort {\n    fn charge(&self);\n}\n",
        ),
        (
            "ports/b.rs",
            "pub trait BillingPort {\n    fn charge(&self);\n}\nuse mcb_domain::repositories::billing::BillingRepository;\n",
        ),
    ]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, SsotViolation::DuplicatePortDeclaration { .. }))
    );
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, SsotViolation::ForbiddenLegacyImport { .. }))
    );
}

#[test]
fn returns_no_violations_for_clean_synthetic_files() {
    let files = synthetic_files(&[
        (
            "ports/repositories/user_repository.rs",
            "pub trait UserRepository {\n    fn find(&self, id: &str);\n}\n",
        ),
        (
            "application/service.rs",
            "use mcb_domain::ports::repositories::user_repository::UserRepository;\n",
        ),
    ]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    assert!(violations.is_empty());
}

#[test]
fn detects_forbidden_legacy_schema_symbol_on_project_schema_struct() {
    let files = synthetic_files(&[(
        "schema/project.rs",
        "pub struct ProjectSchema {
    pub id: String,
}
",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let schema_symbol_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::ForbiddenLegacySchemaSymbol { .. }))
        .collect::<Vec<_>>();

    assert_eq!(schema_symbol_violations.len(), 1);
    match schema_symbol_violations[0] {
        SsotViolation::ForbiddenLegacySchemaSymbol {
            file,
            line,
            symbol_name,
            ..
        } => {
            assert_eq!(file.to_string_lossy(), "schema/project.rs");
            assert_eq!(*line, 1);
            assert_eq!(symbol_name, "ProjectSchema");
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}

#[test]
fn detects_forbidden_legacy_schema_symbol_on_memory_schema_import() {
    let files = synthetic_files(&[(
        "application/use_case.rs",
        "use mcb_domain::MemorySchema;
",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let schema_symbol_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::ForbiddenLegacySchemaSymbol { .. }))
        .collect::<Vec<_>>();

    assert_eq!(schema_symbol_violations.len(), 1);
    match schema_symbol_violations[0] {
        SsotViolation::ForbiddenLegacySchemaSymbol {
            symbol_name, line, ..
        } => {
            assert_eq!(symbol_name, "MemorySchema");
            assert_eq!(*line, 1);
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}

#[test]
fn detects_forbidden_schema_memory_macro_path() {
    let files = synthetic_files(&[(
        "schema/macros.rs",
        "macro_rules! memory_type {
    () => { $crate::schema::memory::ColumnType };
}
",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let macro_path_violations = violations
        .iter()
        .filter(|violation| {
            matches!(
                violation,
                SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(macro_path_violations.len(), 1);
    match macro_path_violations[0] {
        SsotViolation::ForbiddenSchemaMemoryMacroPath {
            file,
            line,
            macro_path,
            ..
        } => {
            assert_eq!(file.to_string_lossy(), "schema/macros.rs");
            assert_eq!(*line, 2);
            assert!(macro_path.contains("$crate::schema::memory::ColumnType"));
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenLegacySchemaSymbol { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}

#[test]
fn detects_forbidden_legacy_schema_import_paths() {
    let files = synthetic_files(&[(
        "application/legacy.rs",
        "use mcb_domain::ProjectSchema;
",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let schema_import_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::ForbiddenLegacySchemaImport { .. }))
        .collect::<Vec<_>>();

    assert_eq!(schema_import_violations.len(), 1);
    match schema_import_violations[0] {
        SsotViolation::ForbiddenLegacySchemaImport {
            file,
            line,
            import_path,
            ..
        } => {
            assert_eq!(file.to_string_lossy(), "application/legacy.rs");
            assert_eq!(*line, 1);
            assert!(import_path.contains("ProjectSchema"));
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenLegacySchemaSymbol { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}

#[test]
fn detects_forbidden_legacy_schema_symbol_on_ddl_generator_struct() {
    let files = synthetic_files(&[(
        "schema/ddl.rs",
        "pub struct MemorySchemaDdlGenerator;
",
    )]);

    let violations = SsotValidator::validate_synthetic_files(&files).unwrap();

    let schema_symbol_violations = violations
        .iter()
        .filter(|violation| matches!(violation, SsotViolation::ForbiddenLegacySchemaSymbol { .. }))
        .collect::<Vec<_>>();

    assert_eq!(schema_symbol_violations.len(), 1);
    match schema_symbol_violations[0] {
        SsotViolation::ForbiddenLegacySchemaSymbol {
            symbol_name, line, ..
        } => {
            assert_eq!(symbol_name, "MemorySchemaDdlGenerator");
            assert_eq!(*line, 1);
        }
        SsotViolation::DuplicatePortDeclaration { .. }
        | SsotViolation::ForbiddenLegacyImport { .. }
        | SsotViolation::ForbiddenSchemaMemoryMacroPath { .. }
        | SsotViolation::ForbiddenLegacySchemaImport { .. } => {
            unreachable!("Unexpected violation variant")
        }
    }
}
