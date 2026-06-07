use regex::Regex;

use crate::Result;

use super::PATTERNS;

pub(crate) fn required_pattern(pattern_id: &str) -> Result<&'static Regex> {
    PATTERNS
        .get(pattern_id)
        .ok_or_else(|| crate::ValidationError::PatternNotFound(pattern_id.to_owned()))
}

pub(crate) fn required_patterns<'a, I>(pattern_ids: I) -> Result<Vec<&'static Regex>>
where
    I: IntoIterator<Item = &'a str>,
{
    pattern_ids.into_iter().map(required_pattern).collect()
}
