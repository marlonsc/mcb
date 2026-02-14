use crate::pattern_registry::required_pattern;
use crate::scan::for_each_rs_under_root;
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;

use super::super::violation::CleanArchitectureViolation;
use super::CleanArchitectureValidator;

impl CleanArchitectureValidator {
    pub(super) fn validate_server_layer_boundaries(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.workspace_root.join(&self.rules.server_path);

        if !server_crate.exists() {
            return Ok(violations);
        }

        let pattern = format!(r"use\s+{}(?:::|;)", self.naming.providers_crate);
        let provider_import_re =
            Regex::new(&pattern).map_err(crate::ValidationError::InvalidRegex)?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &server_crate.join("src"), |path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                if provider_import_re.is_match(line) {
                    violations.push(CleanArchitectureViolation::ServerImportsProviderDirectly {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        import_path: line.trim().to_string(),
                        severity: Severity::Error,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    pub(super) fn validate_handler_injection(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.workspace_root.join(&self.rules.server_path);

        if !server_crate.exists() {
            return Ok(violations);
        }

        let constructor_pattern = r"(\w+)(?:Service|Provider|Repository)(?:Impl)?::new\s*\(";
        let creation_re =
            Regex::new(constructor_pattern).map_err(crate::ValidationError::InvalidRegex)?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        let handlers_dir = self.workspace_root.join(&self.rules.handlers_path);
        if !handlers_dir.exists() {
            return Ok(violations);
        }

        for_each_rs_under_root(&scan_config, &handlers_dir, |path| {
            let content = std::fs::read_to_string(path)?;

            let mut in_test_section = false;
            for (line_num, line) in content.lines().enumerate() {
                if line.contains("#[cfg(test)]") {
                    in_test_section = true;
                    continue;
                }
                if in_test_section || line.contains("#[test]") {
                    continue;
                }

                if let Some(captures) = creation_re.captures(line) {
                    let service_name = captures.get(1).map_or("unknown", |m| m.as_str());
                    violations.push(CleanArchitectureViolation::HandlerCreatesService {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        service_name: service_name.to_string(),
                        context: "Direct creation instead of DI".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    pub(super) fn validate_ca007_infrastructure_concrete_imports(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.workspace_root.join(&self.rules.infrastructure_path);

        if !infra_crate.exists() {
            return Ok(violations);
        }

        let concrete_type_pattern =
            format!(r"use\s+{}::(\w+)::(\w+Impl)", self.naming.application_crate);
        let concrete_type_re =
            Regex::new(&concrete_type_pattern).map_err(crate::ValidationError::InvalidRegex)?;

        let concrete_service_pattern =
            format!(r"use\s+{}::services::(\w+)", self.naming.application_crate);
        let concrete_service_re =
            Regex::new(&concrete_service_pattern).map_err(crate::ValidationError::InvalidRegex)?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &infra_crate.join("src"), |path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//")
                    || trimmed.starts_with("#[test]")
                    || trimmed.starts_with("#[cfg(test)]")
                {
                    continue;
                }

                if let Some(captures) = concrete_type_re.captures(line) {
                    let module = captures.get(1).map_or("?", |m| m.as_str());
                    let concrete_type = captures.get(2).map_or("?", |m| m.as_str());
                    violations.push(
                        CleanArchitectureViolation::InfrastructureImportsConcreteService {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            import_path: format!(
                                "{}::{module}::{concrete_type}",
                                self.naming.application_crate
                            ),
                            concrete_type: concrete_type.to_string(),
                            severity: Severity::Error,
                        },
                    );
                }

                if let Some(captures) = concrete_service_re.captures(line) {
                    let service_name = captures.get(1).map_or("?", |m| m.as_str());
                    if !line.contains("ports::") && !line.contains("dyn ") {
                        violations.push(
                            CleanArchitectureViolation::InfrastructureImportsConcreteService {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                import_path: line.trim().to_string(),
                                concrete_type: service_name.to_string(),
                                severity: Severity::Warning,
                            },
                        );
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    pub(super) fn validate_ca008_application_port_imports(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let app_crate = self.workspace_root.join(&self.rules.application_path);

        if !app_crate.exists() {
            return Ok(violations);
        }

        let ports_dir = self.workspace_root.join(&self.rules.ports_providers_path);
        if !ports_dir.exists() {
            return Ok(violations);
        }

        let local_trait_re = required_pattern("CA002.port_trait_decl")?;

        let reexport_pattern = format!(r"pub\s+use\s+{}::(.*)", self.naming.domain_crate);
        let reexport_re =
            Regex::new(&reexport_pattern).map_err(crate::ValidationError::InvalidRegex)?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &app_crate, |path| {
            let content = std::fs::read_to_string(path)?;
            let has_reexport = reexport_re.is_match(&content);

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                if let Some(captures) = local_trait_re.captures(line) {
                    let trait_name = captures.get(1).map_or("?", |m| m.as_str());

                    if !has_reexport {
                        let relative = path.strip_prefix(&self.workspace_root).unwrap_or(path);
                        violations.push(CleanArchitectureViolation::ApplicationWrongPortImport {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            import_path: format!("{}::{trait_name}", relative.display()),
                            should_be: format!(
                                "{}::ports::providers::{trait_name}",
                                self.naming.domain_crate
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    pub(super) fn validate_ca009_infrastructure_imports_application(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.workspace_root.join(&self.rules.infrastructure_path);

        if !infra_crate.exists() {
            return Ok(violations);
        }

        let app_import_pattern = format!(r"use\s+{}(?:::|;)", self.naming.application_crate);
        let app_import_re =
            Regex::new(&app_import_pattern).map_err(crate::ValidationError::InvalidRegex)?;

        let import_path_pattern = format!(r"use\s+({}::\S+)", self.naming.application_crate);
        let import_path_re =
            Regex::new(&import_path_pattern).map_err(crate::ValidationError::InvalidRegex)?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &infra_crate.join("src"), |path| {
            if self
                .rules
                .composition_root_skip_patterns
                .iter()
                .any(|p| path.to_str().is_some_and(|s| s.contains(p)))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                if app_import_re.is_match(line) {
                    let import_path = import_path_re
                        .captures(line)
                        .and_then(|c| c.get(1))
                        .map_or_else(
                            || self.naming.application_crate.clone(),
                            |m| m.as_str().to_string(),
                        );

                    let suggestion = if import_path.contains("::ports::providers::") {
                        format!(
                            "Import from {} instead: {}",
                            self.naming.domain_crate,
                            import_path
                                .replace(&self.naming.application_crate, &self.naming.domain_crate)
                        )
                    } else if import_path.contains("::services::") {
                        "Services should be injected via DI, not imported. Use Arc<dyn ServiceTrait> in function signatures.".to_string()
                    } else if import_path.contains("::registry::") {
                        format!(
                            "Registry should be accessed from {} via {}, not {}.",
                            self.naming.application_crate,
                            self.naming.providers_crate,
                            self.naming.infrastructure_crate
                        )
                    } else {
                        format!(
                            "{} should NOT depend on {}. Move required traits to {} or refactor to use DI.",
                            self.naming.infrastructure_crate,
                            self.naming.application_crate,
                            self.naming.domain_crate
                        )
                    };

                    violations.push(
                        CleanArchitectureViolation::InfrastructureImportsApplication {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            import_path: import_path.clone(),
                            suggestion,
                            severity: Severity::Error,
                        },
                    );
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }
}
