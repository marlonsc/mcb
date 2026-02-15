use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::scan::{for_each_scan_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::sync::OnceLock;

static IMPL_BLOCK_PATTERN: OnceLock<Regex> = OnceLock::new();
static METHOD_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Verifies that the domain layer contains only trait definitions and data structures, free of implementation logic.
///
/// Ensures that the domain layer remains pure and free of side effects or business logic implementation,
/// permitting only:
/// - Trait definitions.
/// - Struct/enum definitions.
/// - Constructors, accessors, and derived implementations.
pub fn validate_domain_traits_only(
    config: &ValidationConfig,
) -> Result<Vec<OrganizationViolation>> {
    fn is_getter_method(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("&self") && !trimmed.contains("&mut self")
    }
    let mut violations = Vec::new();

    // Pattern for impl blocks with methods
    let impl_block_pattern = IMPL_BLOCK_PATTERN.get_or_init(|| {
        Regex::new(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s*\{").expect("Invalid impl block regex")
    });
    let method_pattern = METHOD_PATTERN.get_or_init(|| {
        Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")
            .expect("Invalid method regex")
    });

    // Allowed method names (constructors, accessors, conversions, simple getters)
    let allowed_methods = [
        "new",
        "default",
        "definition", // Canonical schema factory (CA: data definition, not business logic)
        "tables",
        "fts_def",
        "indexes",
        "foreign_keys",
        "unique_constraints", // Schema builder helpers (data definition)
        "from",
        "into",
        "as_ref",
        "as_mut",
        "clone",
        "fmt",
        "eq",
        "cmp",
        "hash",
        "partial_cmp",
        "is_empty",
        "len",
        "iter",
        "into_iter",
        // Value object utility methods
        "total_changes",
        "from_ast",
        "from_fallback",
        "directory",
        "file",
        "sorted",
        "sort_children",
        // Simple getters that start with common prefixes
    ];
    // Also allow any method starting with common prefixes (factory methods on value objects)
    // Note: These are checked inline below rather than via this array for performance
    let allowed_prefixes = [
        "from_", "into_", "as_", "to_", "get_", "is_", "has_", "with_",
    ];

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, src_dir| {
        // Only check domain crate
        if !src_dir.to_str().is_some_and(|s| s.contains("domain")) {
            return Ok(());
        }

        let Some(path_str) = entry.absolute_path.to_str() else {
            return Ok(());
        };

        // Skip test files
        if is_test_path(path_str) {
            return Ok(());
        }

        // Skip ports (trait definitions expected there)
        if path_str.contains("/ports/") {
            return Ok(());
        }

        let content = std::fs::read_to_string(&entry.absolute_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut in_impl_block = false;
        let mut impl_name = String::new();
        let mut brace_depth = 0;
        let mut impl_start_brace = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Track impl blocks
            if let Some(cap) = impl_block_pattern.captures(line)
                && !trimmed.contains("trait ")
            {
                in_impl_block = true;
                impl_name = cap.get(1).map_or("", |m| m.as_str()).to_owned();
                impl_start_brace = brace_depth;
            }

            brace_depth += i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(0);
            brace_depth -= i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(0);

            if in_impl_block && brace_depth <= impl_start_brace {
                in_impl_block = false;
            }

            if in_impl_block && let Some(cap) = method_pattern.captures(line) {
                let method_name = cap.get(1).map_or("", |m| m.as_str());

                if allowed_methods.contains(&method_name) {
                    continue;
                }

                if allowed_prefixes.iter().any(|p| method_name.starts_with(p)) {
                    continue;
                }

                // Check if this is a getter method (takes &self, returns value, no side effects)
                if is_getter_method(line) {
                    continue;
                }

                // This looks like business logic in domain layer
                violations.push(OrganizationViolation::DomainLayerImplementation {
                    file: entry.absolute_path.clone(),
                    line: line_num + 1,
                    impl_type: "method".to_owned(),
                    type_name: format!("{impl_name}::{method_name}"),
                    severity: Severity::Info,
                });
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
