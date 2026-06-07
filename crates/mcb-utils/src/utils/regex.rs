use regex::Regex;

use crate::error::{Result, UtilsError};

/// Compiles a regex pattern, returning a `UtilsError::Regex` on failure.
///
/// # Errors
/// Returns `UtilsError::Regex` if the pattern is invalid.
pub fn compile_regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| UtilsError::Regex(e.to_string()))
}

/// Compiles a list of regex patterns.
///
/// # Errors
/// Returns `UtilsError::Regex` if any pattern is invalid.
pub fn compile_regexes<'a, I>(patterns: I) -> Result<Vec<Regex>>
where
    I: IntoIterator<Item = &'a str>,
{
    patterns.into_iter().map(compile_regex).collect()
}

/// Compiles a list of regex patterns paired with descriptions.
///
/// # Errors
/// Returns `UtilsError::Regex` if any pattern is invalid.
pub fn compile_regex_pairs<'a>(patterns: &[(&str, &'a str)]) -> Result<Vec<(Regex, &'a str)>> {
    patterns
        .iter()
        .map(|(pattern, desc)| compile_regex(pattern).map(|regex| (regex, *desc)))
        .collect()
}

/// Compiles a list of regex patterns paired with a description and a suggestion string.
///
/// # Errors
/// Returns `UtilsError::Regex` if any pattern is invalid.
pub fn compile_regex_triples<'a>(
    patterns: &[(&str, &'a str, &'a str)],
) -> Result<Vec<(Regex, &'a str, &'a str)>> {
    patterns
        .iter()
        .map(|(pattern, desc, suggestion)| {
            compile_regex(pattern).map(|regex| (regex, *desc, *suggestion))
        })
        .collect()
}
