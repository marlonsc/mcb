use super::violation::OrganizationViolation;
use crate::scan::{for_each_crate_rs_path, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::sync::OnceLock;

static TRAIT_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Verifies that trait definitions are located in the appropriate ports directory.
pub fn validate_trait_placement(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();
    let trait_pattern = TRAIT_PATTERN.get_or_init(|| {
        Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid trait pattern regex")
    });

    // Traits that are OK outside ports (standard patterns)
    let allowed_traits = [
        "Debug",
        "Clone",
        "Default",
        "Display",
        "Error",
        "From",
        "Into",
        "AsRef",
        "Deref",
        "Iterator",
        "Send",
        "Sync",
        "Sized",
        "Copy",
        "Eq",
        "PartialEq",
        "Ord",
        "PartialOrd",
        "Hash",
        "Serialize",
        "Deserialize",
    ];

    // Trait suffixes that are OK in infrastructure (implementation details)
    let allowed_suffixes = [
        "Ext",       // Extension traits
        "Factory",   // Factory patterns
        "Builder",   // Builder patterns
        "Helper",    // Helper traits
        "Internal",  // Internal traits
        "Impl",      // Implementation traits
        "Adapter",   // Adapter-specific traits
        "Handler",   // Handler traits (event handlers, etc.)
        "Listener",  // Event listeners
        "Callback",  // Callback traits
        "Module",    // DI module traits
        "Component", // DI component traits
    ];

    for_each_crate_rs_path(config, |path, _src_dir, crate_name| {
        // Skip domain crate (traits are allowed there)
        if crate_name.contains("domain") {
            return Ok(());
        }

        let Some(path_str) = path.to_str() else {
            return Ok(());
        };

        // Skip if in ports directory (re-exports are OK)
        if path_str.contains("/ports/") {
            return Ok(());
        }

        // Skip DI modules (they often define internal traits)
        if path_str.contains("/di/") {
            return Ok(());
        }

        // Skip test files
        if is_test_path(path_str) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut in_test_module = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Track test module context
            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            // Skip traits in test modules
            if in_test_module {
                continue;
            }

            if let Some(cap) = trait_pattern.captures(line) {
                let trait_name = cap.get(1).map_or("", |m| m.as_str());

                // Skip allowed traits
                if allowed_traits.contains(&trait_name) {
                    continue;
                }

                // Skip internal/private traits (starts with underscore)
                if trait_name.starts_with('_') {
                    continue;
                }

                // Skip traits with allowed suffixes
                if allowed_suffixes
                    .iter()
                    .any(|suffix| trait_name.ends_with(suffix))
                {
                    continue;
                }

                // Skip traits that are clearly internal (private trait declarations)
                if trimmed.starts_with("trait ") && !trimmed.starts_with("pub trait ") {
                    continue;
                }

                // Infrastructure-specific provider traits that are OK outside ports
                // These are implementation details, not domain contracts
                let infra_provider_patterns = [
                    "CacheProvider",      // Caching is infrastructure
                    "HttpClientProvider", // HTTP client is infrastructure
                    "ConfigProvider",     // Config loading is infrastructure
                    "LogProvider",        // Logging is infrastructure
                    "MetricsProvider",    // Metrics is infrastructure
                    "TracingProvider",    // Tracing is infrastructure
                    "StorageProvider",    // Low-level storage is infra
                ];

                // Skip infrastructure-specific providers
                if infra_provider_patterns.contains(&trait_name) {
                    continue;
                }

                // Only flag Provider/Service/Repository traits that look like ports
                if trait_name.contains("Provider")
                    || trait_name.contains("Service")
                    || trait_name.contains("Repository")
                    || trait_name.ends_with("Interface")
                {
                    violations.push(OrganizationViolation::TraitOutsidePorts {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        trait_name: trait_name.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
