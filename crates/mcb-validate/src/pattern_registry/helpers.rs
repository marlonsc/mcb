use regex::Regex;

use crate::Result;

use super::PATTERNS;

pub(crate) fn required_pattern(pattern_id: &str) -> Result<&'static Regex> {
    PATTERNS
        .get(pattern_id)
        .ok_or_else(|| crate::ValidationError::PatternNotFound(pattern_id.to_owned()))
}
