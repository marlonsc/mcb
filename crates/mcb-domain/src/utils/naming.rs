//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Naming convention utilities for the MCB workspace.
//!
//! Provides checks for `CamelCase`, `snake_case`, and `SCREAMING_SNAKE_CASE`.

/// Checks if a name follows CamelCase convention.
#[must_use]
pub fn is_camel_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with uppercase
    let Some(first_char) = name.chars().next() else {
        return false;
    };
    if !first_char.is_ascii_uppercase() {
        return false;
    }

    // No underscores allowed (except at the start for private items, which we skip)
    if name.contains('_') {
        return false;
    }

    // Must have at least one lowercase letter
    name.chars().any(|c| c.is_ascii_lowercase())
}

/// Helper to validate snake-like case conventions.
fn is_valid_snake_case(name: &str, is_uppercase: bool) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must be all lowercase/uppercase (depending on is_uppercase) or underscores or digits
    for c in name.chars() {
        let valid_case = if is_uppercase {
            c.is_ascii_uppercase()
        } else {
            c.is_ascii_lowercase()
        };
        if !valid_case && c != '_' && !c.is_ascii_digit() {
            return false;
        }
    }

    // Can't start with digit
    !name.chars().next().is_some_and(|c| c.is_ascii_digit())
}

/// Checks if a name follows `snake_case` convention.
#[must_use]
pub fn is_snake_case(name: &str) -> bool {
    is_valid_snake_case(name, false)
}

/// Checks if a name follows `SCREAMING_SNAKE_CASE` convention.
#[must_use]
pub fn is_screaming_snake_case(name: &str) -> bool {
    is_valid_snake_case(name, true)
}

/// Extracts the suffix from a file name (part after the last underscore).
#[must_use]
pub fn get_suffix(name: &str) -> &str {
    if let Some(pos) = name.rfind('_') {
        &name[pos..]
    } else {
        ""
    }
}

/// Splits a CamelCase or `PascalCase` string into its constituent words.
#[must_use]
pub fn split_camel_case(s: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            if start < i {
                words.push(&s[start..i]);
            }
            start = i;
        }
    }
    if start < s.len() {
        words.push(&s[start..]);
    }
    words
}
