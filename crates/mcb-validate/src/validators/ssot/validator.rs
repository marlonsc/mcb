use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::traits::validator::Validator;
use crate::traits::violation::Violation;
use crate::{Result, Severity, ValidationConfig};

use super::violation::SsotViolation;

const DECLARATION_PATTERN: &str = r"\bpub\s+(?:trait|struct)\s+([A-Z][A-Za-z0-9_]*)\b";
const LEGACY_IMPORT_PATTERN: &str = r"\b(?:pub\s+)?use\s+mcb_domain::repositories::";
const FORBIDDEN_SCHEMA_SYMBOL: &str = r"\b(ProjectSchema|MemorySchema|MemorySchemaDdlGenerator)\b";
const FORBIDDEN_SCHEMA_MACRO_PATH: &str = r"\$crate::schema::memory::";
const FORBIDDEN_SCHEMA_IMPORT_PATTERN: &str =
    r"\b(?:pub\s+)?use\s+[^;]*\b(ProjectSchema|MemorySchema)\b";
const FORBIDDEN_ROOT_SCHEMA_PATH_PATTERN: &str = r"\bmcb_domain::(Schema|SchemaDdlGenerator|ColumnDef|ColumnType|TableDef|IndexDef|FtsDef|ForeignKeyDef|UniqueConstraintDef|COL_OBSERVATION_TYPE)\b";
const FORBIDDEN_ROOT_SCHEMA_IMPORT_PATTERN: &str = r"\b(?:pub\s+)?use\s+mcb_domain::\{[^}]*\b(Schema|SchemaDdlGenerator|ColumnDef|ColumnType|TableDef|IndexDef|FtsDef|ForeignKeyDef|UniqueConstraintDef|COL_OBSERVATION_TYPE)\b[^}]*\}";

/// Validator for single-source-of-truth invariants.
pub struct SsotValidator {
    config: ValidationConfig,
}

crate::impl_simple_validator_new!(SsotValidator);

impl SsotValidator {
    /// Scan the configured workspace and return SSOT violations.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or regex compilation fails.
    pub fn validate_all(&self) -> Result<Vec<SsotViolation>> {
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
    ///
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

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut declaration_locations: BTreeMap<String, Vec<(PathBuf, usize)>> = BTreeMap::new();
        let mut violations = Vec::new();

        for (path, content) in &files {
            for (line_index, line) in content.lines().enumerate() {
                for cap in declaration_pattern.captures_iter(line) {
                    if let Some(name_match) = cap.get(1) {
                        declaration_locations
                            .entry(name_match.as_str().to_owned())
                            .or_default()
                            .push((path.clone(), line_index + 1));
                    }
                }

                if legacy_import_pattern.is_match(line) {
                    violations.push(SsotViolation::ForbiddenLegacyImport {
                        file: path.clone(),
                        line: line_index + 1,
                        import_path: line.trim().to_owned(),
                        severity: Severity::Error,
                    });
                }

                for cap in forbidden_schema_symbol_pattern.captures_iter(line) {
                    if let Some(symbol_name_match) = cap.get(1) {
                        violations.push(SsotViolation::ForbiddenLegacySchemaSymbol {
                            file: path.clone(),
                            line: line_index + 1,
                            symbol_name: symbol_name_match.as_str().to_owned(),
                            severity: Severity::Error,
                        });
                    }
                }

                if forbidden_schema_macro_path_pattern.is_match(line) {
                    violations.push(SsotViolation::ForbiddenSchemaMemoryMacroPath {
                        file: path.clone(),
                        line: line_index + 1,
                        macro_path: line.trim().to_owned(),
                        severity: Severity::Error,
                    });
                }

                if forbidden_schema_import_pattern.is_match(line) {
                    violations.push(SsotViolation::ForbiddenLegacySchemaImport {
                        file: path.clone(),
                        line: line_index + 1,
                        import_path: line.trim().to_owned(),
                        severity: Severity::Error,
                    });
                }

                if forbidden_root_schema_import_pattern.is_match(line) {
                    violations.push(SsotViolation::ForbiddenRootSchemaPath {
                        file: path.clone(),
                        line: line_index + 1,
                        path: line.trim().to_owned(),
                        severity: Severity::Error,
                    });
                }

                for cap in forbidden_root_schema_path_pattern.captures_iter(line) {
                    if let Some(path_match) = cap.get(0) {
                        violations.push(SsotViolation::ForbiddenRootSchemaPath {
                            file: path.clone(),
                            line: line_index + 1,
                            path: path_match.as_str().to_owned(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

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

        Ok(violations)
    }
}

impl Validator for SsotValidator {
    fn name(&self) -> &'static str {
        "ssot"
    }

    fn description(&self) -> &'static str {
        "Detects duplicate declarations and forbidden legacy schema/repository references"
    }

    fn enabled_by_default(&self) -> bool {
        false
    }

    fn validate(&self, _config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
