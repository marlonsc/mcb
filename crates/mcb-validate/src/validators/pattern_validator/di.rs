use std::path::Path;

use regex::Regex;

use super::violation::PatternViolation;
use crate::traits::violation::Severity;

/// Checks for Arc<Concrete> usage in a single file.
pub fn check_arc_usage(
    path: &Path,
    content: &str,
    arc_pattern: &Regex,
    allowed_concrete: &[String],
    provider_traits: &[String],
) -> Vec<PatternViolation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Check for ignore hints
        let has_ignore_hint = line.contains("mcb-validate-ignore: admin_service_concrete_type");

        for cap in arc_pattern.captures_iter(line) {
            let type_name = cap.get(1).map_or("", |m| m.as_str());

            // Skip allowed concrete types
            if allowed_concrete.iter().any(|s| s == type_name) {
                continue;
            }

            // Skip if already using dyn (handled by different pattern)
            if line.contains(&format!("Arc<dyn {type_name}")) {
                continue;
            }

            // Skip decorator pattern: Arc<Type<T>> (generic wrapper types)
            if line.contains(&format!("Arc<{type_name}<")) {
                continue;
            }

            // Check if type name ends with a provider trait suffix
            let is_likely_provider = provider_traits
                .iter()
                .any(|suffix| type_name.ends_with(suffix));

            // Also check for common service implementation patterns
            let is_impl_suffix = type_name.ends_with("Impl")
                || type_name.ends_with("Implementation")
                || type_name.ends_with("Adapter");

            if is_likely_provider || is_impl_suffix {
                // Skip if ignore hint is present
                if has_ignore_hint {
                    continue;
                }

                let trait_name = if is_impl_suffix {
                    type_name
                        .trim_end_matches("Impl")
                        .trim_end_matches("Implementation")
                        .trim_end_matches("Adapter")
                } else {
                    type_name
                };

                violations.push(PatternViolation::ConcreteTypeInDi {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    concrete_type: format!("Arc<{type_name}>"),
                    suggestion: format!("Arc<dyn {trait_name}>"),
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}
