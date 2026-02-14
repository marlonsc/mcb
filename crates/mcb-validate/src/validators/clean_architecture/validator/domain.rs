use crate::pattern_registry::required_pattern;
use crate::scan::for_each_rs_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::super::violation::CleanArchitectureViolation;
use super::CleanArchitectureValidator;

impl CleanArchitectureValidator {
    /// Validate entities have identity fields
    pub(super) fn validate_entity_identity(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let domain_crate = self.workspace_root.join(&self.rules.domain_path);

        if !domain_crate.exists() {
            return Ok(violations);
        }

        let entities_dir = self.workspace_root.join(&self.rules.entities_path);
        if !entities_dir.exists() {
            return Ok(violations);
        }

        // Look for struct definitions that should have id fields
        let struct_re = required_pattern("CA001.pub_struct_brace")?;
        let id_field_re = required_pattern("CA001.id_field")?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &entities_dir, |path| {
            // Skip mod.rs files
            if path.file_name().is_some_and(|n| n == "mod.rs") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(captures) = struct_re.captures(line) {
                    let struct_name = captures.get(1).map_or("unknown", |m| m.as_str());

                    // Skip if not an entity (e.g., helper structs, value objects)
                    // Value Objects like *Changes don't need identity
                    if self
                        .rules
                        .identity_skip_suffixes
                        .iter()
                        .any(|s| struct_name.ends_with(s))
                    {
                        continue;
                    }

                    // Look ahead for id field in struct definition
                    let mut has_id = false;
                    let mut brace_count = 0;
                    let mut started = false;

                    for check_line in lines.iter().skip(line_num) {
                        if check_line.contains('{') {
                            brace_count += check_line.matches('{').count();
                            started = true;
                        }
                        if check_line.contains('}') {
                            brace_count -= check_line.matches('}').count();
                        }

                        if id_field_re.is_match(check_line) {
                            has_id = true;
                            break;
                        }

                        if started && brace_count == 0 {
                            break;
                        }
                    }

                    if !has_id {
                        violations.push(CleanArchitectureViolation::EntityMissingIdentity {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            entity_name: struct_name.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Validate value objects are immutable
    pub(super) fn validate_value_object_immutability(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let domain_crate = self.workspace_root.join(&self.rules.domain_path);

        if !domain_crate.exists() {
            return Ok(violations);
        }

        let vo_dir = self.workspace_root.join(&self.rules.vo_path);
        if !vo_dir.exists() {
            return Ok(violations);
        }

        // Look for &mut self methods in value objects
        let impl_re = required_pattern("CA001.impl_block")?;
        let mut_method_re = required_pattern("CA001.mut_self_method")?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &vo_dir, |path| {
            // Skip mod.rs files
            if path.file_name().is_some_and(|n| n == "mod.rs") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut current_impl: Option<String> = None;
            let mut brace_depth = 0;

            for (line_num, line) in lines.iter().enumerate() {
                // Track impl blocks
                if let Some(captures) = impl_re.captures(line) {
                    current_impl = captures.get(1).map(|m| m.as_str().to_string());
                }

                // Track brace depth for impl scope
                brace_depth += line.matches('{').count();
                brace_depth -= line.matches('}').count();

                if brace_depth == 0 {
                    current_impl = None;
                }

                // Check for mutable methods
                if let Some(ref vo_name) = current_impl
                    && let Some(captures) = mut_method_re.captures(line)
                {
                    let method_name = captures.get(1).map_or("?", |m| m.as_str());

                    // Allow some standard mutable methods
                    if !self
                        .rules
                        .allowed_mutable_prefixes
                        .iter()
                        .any(|p| method_name.starts_with(p))
                    {
                        continue;
                    }

                    violations.push(CleanArchitectureViolation::ValueObjectMutable {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        vo_name: vo_name.clone(),
                        method_name: method_name.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }
}
