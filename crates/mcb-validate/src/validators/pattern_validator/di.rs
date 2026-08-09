//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use regex::Regex;

use super::violation::PatternViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::constants::validate::{COMMENT_PREFIX, DI_IMPL_SUFFIXES, VALIDATE_IGNORE_PREFIX};

/// Checks for Arc<Concrete> usage in a single file.
pub fn check_arc_usage(
    path: &Path,
    content: &str,
    arc_pattern: &Regex,
    allowed_concrete: &[String],
    provider_traits: &[String],
) -> Vec<PatternViolation> {
    let mut violations = Vec::new();
    let ignore_hint = format!("{VALIDATE_IGNORE_PREFIX}admin_service_concrete_type");

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(COMMENT_PREFIX) || line.contains(&ignore_hint) {
            continue;
        }
        collect_arc_violations(
            path,
            line,
            line_num,
            arc_pattern,
            allowed_concrete,
            provider_traits,
            &mut violations,
        );
    }
    violations
}

/// Resolve the trait name that should back `Arc<dyn ..>` for a concrete DI type,
/// or `None` if `type_name` is not a recognizable implementation type.
fn normalized_trait_name(type_name: &str, provider_traits: &[String]) -> Option<String> {
    if provider_traits
        .iter()
        .any(|suffix| type_name.ends_with(suffix))
    {
        return Some(type_name.to_owned());
    }

    let (name, changed) =
        DI_IMPL_SUFFIXES
            .iter()
            .fold((type_name.to_owned(), false), |(name, changed), suffix| {
                let next = name.trim_end_matches(suffix).to_owned();
                (next.clone(), changed || next != name)
            });
    changed.then_some(name)
}

/// Push a `ConcreteTypeInDi` violation for each `Arc<Concrete>` on `line` that
/// should instead use `Arc<dyn Trait>`.
fn collect_arc_violations(
    path: &Path,
    line: &str,
    line_num: usize,
    arc_pattern: &Regex,
    allowed_concrete: &[String],
    provider_traits: &[String],
    violations: &mut Vec<PatternViolation>,
) {
    for cap in arc_pattern.captures_iter(line) {
        let type_name = cap.get(1).map_or("", |m| m.as_str());

        let skip = allowed_concrete.iter().any(|s| s == type_name)
            || line.contains(&format!("Arc<dyn {type_name}"))
            || line.contains(&format!("Arc<{type_name}<"));
        if skip {
            continue;
        }

        if let Some(trait_name) = normalized_trait_name(type_name, provider_traits) {
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
