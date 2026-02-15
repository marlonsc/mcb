//! Pattern Registry Module
//!
//! Provides centralized regex pattern management loaded from YAML rules.
//! Patterns are compiled once at startup and accessed via a global registry.

mod registry;

use regex::Regex;

use crate::Result;

pub use registry::{PATTERNS, PatternRegistry, default_rules_dir};

/// Function to get a required pattern by ID
pub(crate) fn required_pattern(pattern_id: &str) -> Result<&'static Regex> {
    PATTERNS
        .get(pattern_id)
        .ok_or_else(|| crate::ValidationError::PatternNotFound(pattern_id.to_owned()))
}

/// Function to get multiple required patterns by ID
pub(crate) fn required_patterns<'a, I>(pattern_ids: I) -> Result<Vec<&'static Regex>>
where
    I: IntoIterator<Item = &'a str>,
{
    pattern_ids.into_iter().map(required_pattern).collect()
}

pub(crate) fn compile_regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(crate::ValidationError::InvalidRegex)
}

pub(crate) fn compile_regexes<'a, I>(patterns: I) -> Result<Vec<Regex>>
where
    I: IntoIterator<Item = &'a str>,
{
    patterns.into_iter().map(compile_regex).collect()
}

/// Compiles a list of regex patterns paired with descriptions.
///
/// # Errors
///
/// Returns an error if any regex pattern fails to compile.
pub fn compile_regex_pairs<'a>(patterns: &[(&str, &'a str)]) -> Result<Vec<(Regex, &'a str)>> {
    patterns
        .iter()
        .map(|(pattern, desc)| compile_regex(pattern).map(|regex| (regex, *desc)))
        .collect()
}

pub(crate) fn compile_regex_triples<'a>(
    patterns: &[(&str, &'a str, &'a str)],
) -> Result<Vec<(Regex, &'a str, &'a str)>> {
    patterns
        .iter()
        .map(|(pattern, desc, suggestion)| {
            compile_regex(pattern).map(|regex| (regex, *desc, *suggestion))
        })
        .collect()
}
