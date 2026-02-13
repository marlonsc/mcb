//! Clean Architecture validator implementation
// TODO(QUAL004): File too large (604 lines, max: 500).
// Consider splitting into smaller modules (boundaries, injection, entities, etc.).

use std::path::PathBuf;

use super::violation::CleanArchitectureViolation;
use crate::config::CleanArchitectureRulesConfig;
use crate::pattern_registry::PATTERNS;
use crate::scan::for_each_rs_under_root;
use crate::violation_trait::Violation;
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;

/// Clean Architecture validator
pub struct CleanArchitectureValidator {
    workspace_root: PathBuf,
    rules: CleanArchitectureRulesConfig,
    naming: crate::config::NamingRulesConfig,
}

impl CleanArchitectureValidator {
    /// Create a new architecture validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(
            &ValidationConfig::new(root),
            &file_config.rules.clean_architecture,
            &file_config.rules.naming,
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        config: &ValidationConfig,
        rules: &CleanArchitectureRulesConfig,
        naming: &crate::config::NamingRulesConfig,
    ) -> Self {
        Self {
            workspace_root: config.workspace_root.clone(),
            rules: rules.clone(),
            naming: naming.clone(),
        }
    }

    /// Run all architecture validations (returns typed violations)
    pub fn validate_all(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_server_layer_boundaries()?);
        violations.extend(self.validate_handler_injection()?);
        violations.extend(self.validate_entity_identity()?);
        violations.extend(self.validate_value_object_immutability()?);
        // ADR-029: Hexagonal architecture validations
        violations.extend(self.validate_ca007_infrastructure_concrete_imports()?);
        violations.extend(self.validate_ca008_application_port_imports()?);
        // CA009: Infrastructure should NOT import from Application layer
        violations.extend(self.validate_ca009_infrastructure_imports_application()?);
        Ok(violations)
    }

    /// Run all validations (returns boxed violations for Validator trait)
    fn validate_boxed(&self) -> Result<Vec<Box<dyn Violation>>> {
        let typed_violations = self.validate_all()?;
        Ok(typed_violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}

impl crate::validator_trait::Validator for CleanArchitectureValidator {
    fn name(&self) -> &'static str {
        "clean_architecture"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture compliance: layer boundaries, DI patterns, entity identity, value object immutability"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        self.validate_boxed().map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl CleanArchitectureValidator {
    /// Validate server layer doesn't import providers directly
    fn validate_server_layer_boundaries(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.workspace_root.join(&self.rules.server_path);

        if !server_crate.exists() {
            return Ok(violations);
        }

        // Dynamically construct regex based on provider crate name
        let pattern = format!(r"use\s+{}(?:::|;)", self.naming.providers_crate);
        let provider_import_re = Regex::new(&pattern).map_err(|e| {
            crate::ValidationError::InvalidRegex(format!("CA001.provider_import: {e}"))
        })?;
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

    /// Validate handlers use dependency injection
    fn validate_handler_injection(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.workspace_root.join(&self.rules.server_path);

        if !server_crate.exists() {
            return Ok(violations);
        }

        // Patterns for direct service creation
        let constructor_pattern = r"(\w+)(?:Service|Provider|Repository)(?:Impl)?::new\s*\(";
        let creation_re = Regex::new(constructor_pattern)
            .map_err(|e| crate::ValidationError::InvalidRegex(format!("Service Creation: {e}")))?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        let handlers_dir = self.workspace_root.join(&self.rules.handlers_path);
        if !handlers_dir.exists() {
            return Ok(violations);
        }

        for_each_rs_under_root(&scan_config, &handlers_dir, |path| {
            let content = std::fs::read_to_string(path)?;

            let mut in_test_section = false;
            for (line_num, line) in content.lines().enumerate() {
                // Skip inline/unit-test code sections (typically below #[cfg(test)]).
                // This avoids false positives from mock/test-only constructors in handlers.
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

    /// Validate entities have identity fields
    fn validate_entity_identity(&self) -> Result<Vec<CleanArchitectureViolation>> {
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
        let struct_re = PATTERNS
            .get("CA001.pub_struct_brace")
            // TODO(QUAL002): expect() in production. Use ? or handle error.
            .expect("Pattern CA001.pub_struct_brace not found");
        let id_field_re = PATTERNS
            .get("CA001.id_field")
            // TODO(QUAL002): expect() in production. Use ? or handle error.
            .expect("Pattern CA001.id_field not found");
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
    fn validate_value_object_immutability(&self) -> Result<Vec<CleanArchitectureViolation>> {
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
        let impl_re = PATTERNS
            .get("CA001.impl_block")
            // TODO(QUAL002): expect() in production. Use ? or handle error.
            .expect("Pattern CA001.impl_block not found");
        let mut_method_re = PATTERNS
            .get("CA001.mut_self_method")
            // TODO(QUAL002): expect() in production. Use ? or handle error.
            .expect("Pattern CA001.mut_self_method not found");
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
                    // TODO(QUAL001): unwrap() in production. Use ? or match.
                    current_impl = Some(captures.get(1).map(|m| m.as_str().to_string()).unwrap());
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

    /// CA007: Validate infrastructure layer doesn't import concrete types from application
    ///
    /// Infrastructure should only import trait interfaces (ports), not concrete implementations.
    /// This prevents tight coupling and maintains proper dependency direction.
    fn validate_ca007_infrastructure_concrete_imports(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.workspace_root.join(&self.rules.infrastructure_path);

        if !infra_crate.exists() {
            return Ok(violations);
        }

        // Patterns for concrete types that should NOT be imported
        // These match patterns like: use mcb_application::services::ContextServiceImpl
        // Patterns for concrete types that should NOT be imported
        let concrete_type_pattern =
            format!(r"use\s+{}::(\w+)::(\w+Impl)", self.naming.application_crate);
        let concrete_type_re = Regex::new(&concrete_type_pattern).map_err(|e| {
            crate::ValidationError::InvalidRegex(format!("CA002.app_impl_import: {e}"))
        })?;

        // Also catch any concrete service imports
        let concrete_service_pattern =
            format!(r"use\s+{}::services::(\w+)", self.naming.application_crate);
        let concrete_service_re = Regex::new(&concrete_service_pattern).map_err(|e| {
            crate::ValidationError::InvalidRegex(format!("CA002.app_service_import: {e}"))
        })?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &infra_crate.join("src"), |path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                // Skip comments and test code
                let trimmed = line.trim();
                if trimmed.starts_with("//")
                    || trimmed.starts_with("#[test]")
                    || trimmed.starts_with("#[cfg(test)]")
                {
                    continue;
                }

                // Check for Impl types
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

                // Check for direct service imports (non-trait)
                if let Some(captures) = concrete_service_re.captures(line) {
                    let service_name = captures.get(1).map_or("?", |m| m.as_str());
                    // Allow trait imports (ports)
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

    /// CA008: Validate application layer imports ports from mcb-domain, not locally
    ///
    /// Application layer should not define provider traits locally; they must be
    /// imported from mcb-domain to maintain single source of truth.
    fn validate_ca008_application_port_imports(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let app_crate = self.workspace_root.join(&self.rules.application_path);

        if !app_crate.exists() {
            return Ok(violations);
        }

        let ports_dir = self.workspace_root.join(&self.rules.ports_providers_path);
        if !ports_dir.exists() {
            return Ok(violations);
        }

        // Patterns to detect local trait definitions (violations)
        let local_trait_re = PATTERNS
            .get("CA002.port_trait_decl")
            // TODO(QUAL002): expect() in production. Use ? or handle error.
            .expect("Pattern CA002.port_trait_decl not found");

        // Pattern for allowed re-exports from mcb-domain
        let reexport_pattern = format!(r"pub\s+use\s+{}::(.*)", self.naming.domain_crate);
        let reexport_re = Regex::new(&reexport_pattern).map_err(|e| {
            crate::ValidationError::InvalidRegex(format!("CA002.domain_reexport: {e}"))
        })?;
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

    /// CA009: Validate infrastructure layer does NOT import from application layer
    ///
    /// Per Clean Architecture, the dependency flow should be:
    /// ```text
    /// mcb-server → mcb-infrastructure → mcb-domain
    ///                     ↓                  ↑
    ///               mcb-providers ────→ mcb-application
    /// ```
    ///
    /// Infrastructure should only depend on Domain, NOT Application.
    /// This prevents circular dependencies and maintains proper layering.
    fn validate_ca009_infrastructure_imports_application(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.workspace_root.join(&self.rules.infrastructure_path);

        if !infra_crate.exists() {
            return Ok(violations);
        }

        // Pattern for any import from mcb_application
        let app_import_pattern = format!(r"use\s+{}(?:::|;)", self.naming.application_crate);
        let app_import_re = Regex::new(&app_import_pattern)
            .map_err(|e| crate::ValidationError::InvalidRegex(format!("CA009.app_import: {e}")))?;

        // Extract specific import path
        let import_path_pattern = format!(r"use\s+({}::\S+)", self.naming.application_crate);
        let import_path_re = Regex::new(&import_path_pattern).map_err(|e| {
            crate::ValidationError::InvalidRegex(format!("CA009.app_import_path: {e}"))
        })?;
        let scan_config = ValidationConfig::new(self.workspace_root.clone());

        for_each_rs_under_root(&scan_config, &infra_crate.join("src"), |path| {
            // Skip composition root (di/ directory) - allowed to import application layer
            // for wiring up dependencies. This is the standard Clean Architecture exception
            // where the composition root needs to know about all layers to assemble them.
            if self
                .rules
                .composition_root_skip_patterns
                .iter()
                .any(|p| path.to_string_lossy().contains(p))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                // Skip comments
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                // Check for mcb_application imports
                if app_import_re.is_match(line) {
                    let import_path = import_path_re
                        .captures(line)
                        .and_then(|c| c.get(1))
                        .map_or_else(
                            || self.naming.application_crate.clone(),
                            |m| m.as_str().to_string(),
                        );

                    // Determine suggestion based on what's being imported
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
