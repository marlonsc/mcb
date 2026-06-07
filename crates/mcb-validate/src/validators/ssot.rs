//! Single Source of Truth (SSOT) validation module
//!
//! **Documentation**: [`docs/modules/validate.md#single-source-of-truth-ssot`](../../../../docs/modules/validate.md#single-source-of-truth-ssot)

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};
use mcb_domain::ports::validation::ViolationCategory;

crate::define_validator! {
    name: "ssot",
    description: "Detects duplicate declarations and forbidden legacy schema/repository references",

    /// Validator for single-source-of-truth invariants.
    pub struct SsotValidator {
        config: ValidationConfig,
    }

    violations: dynamic_severity, ViolationCategory::Organization,
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
        #[doc = "Forbidden raw ID field type in domain entities/value objects."]
        #[violation(
            id = "SSOT007",
            severity = Error,
            message = "Forbidden raw ID field type: {file}:{line} uses '{field_type}' for '{field_name}'",
            suggestion = "Use a strong-typed ID from mcb_domain::value_objects::ids via define_id!"
        )]
        ForbiddenRawIdFieldType {
            file: PathBuf,
            line: usize,
            field_name: String,
            field_type: String,
            severity: Severity,
        },
    }

    checks: [
        run_all_checks => Self::analyze_workspace
    ]
}

const DECLARATION_PATTERN: &str = r"\bpub\s+(?:trait|struct)\s+([A-Z][A-Za-z0-9_]*)\b";
const LEGACY_IMPORT_PATTERN: &str = r"\b(?:pub\s+)?use\s+mcb_domain::repositories::";
const FORBIDDEN_SCHEMA_SYMBOL: &str = r"\b(ProjectSchema|MemorySchema|MemorySchemaDdlGenerator)\b";
const FORBIDDEN_SCHEMA_MACRO_PATH: &str = r"\$crate::schema::memory::";
const FORBIDDEN_SCHEMA_IMPORT_PATTERN: &str =
    r"\b(?:pub\s+)?use\s+[^;]*\b(ProjectSchema|MemorySchema)\b";
const FORBIDDEN_ROOT_SCHEMA_PATH_PATTERN: &str = r"\bmcb_domain::(Schema|SchemaDdlGenerator|ColumnDef|ColumnType|TableDef|IndexDef|FtsDef|ForeignKeyDef|UniqueConstraintDef|COL_OBSERVATION_TYPE)\b";
const FORBIDDEN_ROOT_SCHEMA_IMPORT_PATTERN: &str = r"\b(?:pub\s+)?use\s+mcb_domain::\{[^}]*\b(Schema|SchemaDdlGenerator|ColumnDef|ColumnType|TableDef|IndexDef|FtsDef|ForeignKeyDef|UniqueConstraintDef|COL_OBSERVATION_TYPE)\b[^}]*\}";
const FORBIDDEN_RAW_ID_FIELD_PATTERN: &str = r"\bpub\s+([a-z_][a-z0-9_]*)\s*:\s*(String|Uuid)\b";

crate::impl_simple_validator_new!(SsotValidator);

impl SsotValidator {
    /// Overrides the generated `validate_all` method because it needs custom file gathering.
    ///
    /// # Errors
    /// Returns an error if workspace scanning or regex compilation fails.
    pub fn analyze_workspace(&self) -> Result<Vec<SsotViolation>> {
        let mut files = Vec::new();

        for_each_scan_file(
            &self.config,
            Some(LanguageId::Rust),
            false,
            |entry, _candidate_src_dir| {
                let content = std::fs::read_to_string(&entry.absolute_path)?;
                files.push((entry.absolute_path.clone(), content));
                Ok(())
            },
        )?;

        Self::analyze_files(files)
    }

    /// Validate an in-memory synthetic file map (used by unit tests).
    ///
    /// # Errors
    /// Returns an error if regex compilation fails.
    pub fn validate_synthetic_files(files: &HashMap<String, String>) -> Result<Vec<SsotViolation>> {
        let synthetic_files = files
            .iter()
            .map(|(path, content)| (PathBuf::from(path), content.clone()))
            .collect::<Vec<_>>();
        Self::analyze_files(synthetic_files)
    }

    fn analyze_files(mut files: Vec<(PathBuf, String)>) -> Result<Vec<SsotViolation>> {
        let declaration_pattern = compile_regex(DECLARATION_PATTERN)?;
        let legacy_import_pattern = compile_regex(LEGACY_IMPORT_PATTERN)?;
        let forbidden_schema_symbol_pattern = compile_regex(FORBIDDEN_SCHEMA_SYMBOL)?;
        let forbidden_schema_macro_path_pattern = compile_regex(FORBIDDEN_SCHEMA_MACRO_PATH)?;
        let forbidden_schema_import_pattern = compile_regex(FORBIDDEN_SCHEMA_IMPORT_PATTERN)?;
        let forbidden_root_schema_path_pattern = compile_regex(FORBIDDEN_ROOT_SCHEMA_PATH_PATTERN)?;
        let forbidden_root_schema_import_pattern =
            compile_regex(FORBIDDEN_ROOT_SCHEMA_IMPORT_PATTERN)?;
        let forbidden_raw_id_field_pattern = compile_regex(FORBIDDEN_RAW_ID_FIELD_PATTERN)?;

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut declaration_locations: BTreeMap<String, Vec<(PathBuf, usize)>> = BTreeMap::new();
        let mut violations = Vec::new();

        for (path, content) in &files {
            for (line_index, line) in content.lines().enumerate() {
                let line_number = line_index + 1;
                record_declarations(
                    &mut declaration_locations,
                    &declaration_pattern,
                    line,
                    path,
                    line_number,
                );

                push_line_match(
                    &mut violations,
                    &legacy_import_pattern,
                    line,
                    |import_path| SsotViolation::ForbiddenLegacyImport {
                        file: path.clone(),
                        line: line_number,
                        import_path,
                        severity: Severity::Error,
                    },
                );

                push_capture_group(
                    &mut violations,
                    &forbidden_schema_symbol_pattern,
                    line,
                    1,
                    |symbol_name| SsotViolation::ForbiddenLegacySchemaSymbol {
                        file: path.clone(),
                        line: line_number,
                        symbol_name: symbol_name.to_owned(),
                        severity: Severity::Error,
                    },
                );

                push_line_match(
                    &mut violations,
                    &forbidden_schema_macro_path_pattern,
                    line,
                    |macro_path| SsotViolation::ForbiddenSchemaMemoryMacroPath {
                        file: path.clone(),
                        line: line_number,
                        macro_path,
                        severity: Severity::Error,
                    },
                );

                push_line_match(
                    &mut violations,
                    &forbidden_schema_import_pattern,
                    line,
                    |import_path| SsotViolation::ForbiddenLegacySchemaImport {
                        file: path.clone(),
                        line: line_number,
                        import_path,
                        severity: Severity::Error,
                    },
                );

                push_line_match(
                    &mut violations,
                    &forbidden_root_schema_import_pattern,
                    line,
                    |path_text| SsotViolation::ForbiddenRootSchemaPath {
                        file: path.clone(),
                        line: line_number,
                        path: path_text,
                        severity: Severity::Error,
                    },
                );

                push_capture_group(
                    &mut violations,
                    &forbidden_root_schema_path_pattern,
                    line,
                    0,
                    |path_text| SsotViolation::ForbiddenRootSchemaPath {
                        file: path.clone(),
                        line: line_number,
                        path: path_text.to_owned(),
                        severity: Severity::Error,
                    },
                );

                if is_domain_model_file(path) {
                    push_raw_id_field_violations(
                        &mut violations,
                        &forbidden_raw_id_field_pattern,
                        path,
                        line,
                        line_number,
                    );
                }
            }
        }

        push_duplicate_declarations(&mut violations, declaration_locations);

        Ok(violations)
    }
}

fn is_domain_model_file(path: &std::path::Path) -> bool {
    let path = path.to_string_lossy().replace('\\', "/");
    path.contains("mcb-domain/src/entities/") || path.contains("mcb-domain/src/value_objects/")
}

fn push_line_match(
    violations: &mut Vec<SsotViolation>,
    pattern: &regex::Regex,
    line: &str,
    make: impl Fn(String) -> SsotViolation,
) {
    if pattern.is_match(line) {
        violations.push(make(line.trim().to_owned()));
    }
}

fn push_capture_group(
    violations: &mut Vec<SsotViolation>,
    pattern: &regex::Regex,
    line: &str,
    group: usize,
    make: impl Fn(&str) -> SsotViolation,
) {
    for cap in pattern.captures_iter(line) {
        if let Some(m) = cap.get(group) {
            violations.push(make(m.as_str()));
        }
    }
}

fn push_duplicate_declarations(
    violations: &mut Vec<SsotViolation>,
    declaration_locations: BTreeMap<String, Vec<(PathBuf, usize)>>,
) {
    for (declaration_name, locations) in declaration_locations {
        let unique_paths = locations
            .iter()
            .map(|(path, _line)| path.clone())
            .collect::<HashSet<_>>();

        if unique_paths.len() < 2 {
            continue;
        }

        let mut sorted_locations = locations;
        sorted_locations.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let Some((file, line)) = sorted_locations.first().cloned() else {
            continue;
        };

        let locations_text = sorted_locations
            .iter()
            .map(|(path, location_line)| format!("{}:{location_line}", path.display()))
            .collect::<Vec<_>>()
            .join(", ");

        violations.push(SsotViolation::DuplicatePortDeclaration {
            file,
            line,
            declaration_name,
            locations: locations_text,
            severity: Severity::Error,
        });
    }
}

fn record_declarations(
    declaration_locations: &mut BTreeMap<String, Vec<(PathBuf, usize)>>,
    declaration_pattern: &regex::Regex,
    line: &str,
    path: &Path,
    line_number: usize,
) {
    for cap in declaration_pattern.captures_iter(line) {
        if let Some(name_match) = cap.get(1) {
            declaration_locations
                .entry(name_match.as_str().to_owned())
                .or_default()
                .push((path.to_path_buf(), line_number));
        }
    }
}

fn push_raw_id_field_violations(
    violations: &mut Vec<SsotViolation>,
    forbidden_raw_id_field_pattern: &regex::Regex,
    path: &Path,
    line: &str,
    line_number: usize,
) {
    for cap in forbidden_raw_id_field_pattern.captures_iter(line) {
        let Some(field_name_match) = cap.get(1) else {
            continue;
        };
        let Some(field_type_match) = cap.get(2) else {
            continue;
        };

        if !field_name_match.as_str().ends_with("id") {
            continue;
        }

        violations.push(SsotViolation::ForbiddenRawIdFieldType {
            file: path.to_path_buf(),
            line: line_number,
            field_name: field_name_match.as_str().to_owned(),
            field_type: field_type_match.as_str().to_owned(),
            severity: Severity::Error,
        });
    }
}
