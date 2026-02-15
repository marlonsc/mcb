use crate::error::{Error, Result};
use regex::Regex;
use std::path::PathBuf;

/// Record of a discovered function.
#[derive(Debug, Clone)]
pub struct FunctionRecord {
    /// Path to the file containing the function.
    pub file: PathBuf,
    /// Name of the function.
    pub name: String,
    /// Line number where the function starts.
    pub line: usize,
    /// Cyclomatic complexity score of the function.
    pub complexity: u32,
}

/// Computes cyclomatic complexity using control flow keyword counting.
///
/// Returns 1 + count of branching constructs (if, for, while, loop, match, &&, ||).
pub fn compute_complexity_score(content: &str, start_pos: usize) -> Result<u32> {
    let body = extract_function_body(content, start_pos).unwrap_or_default();
    let re = Regex::new(r"\b(if|for|while|loop|match)\b|&&|\|\|")
        .map_err(|e| Error::invalid_argument(format!("invalid complexity regex: {e}")))?;
    let count = re.find_iter(&body).count() as u32;
    Ok(1 + count)
}

/// Extracts the function body by balancing braces.
///
/// Returns the content within the function body, including the braces.
pub fn extract_function_body(content: &str, start_pos: usize) -> Option<String> {
    let after_start = &content[start_pos..];
    let brace_index = after_start.find('{')?;
    let body_start = start_pos + brace_index;

    let bytes = content.as_bytes();
    let mut depth = 0_i32;
    let mut i = body_start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(content[body_start..=i].to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Checks if a symbol should be exempt from dead code detection (e.g. main, tests).
pub fn is_exempt_symbol(name: &str) -> bool {
    matches!(name, "main") || name.starts_with("test_")
}

/// Counts occurrences of symbols across a set of file contents.
///
/// Returns a map of symbol -> count.
pub fn count_symbol_occurrences(
    file_contents: &[String],
    symbols: &[String],
) -> Result<std::collections::HashMap<String, usize>> {
    let mut map = std::collections::HashMap::new();
    for symbol in symbols {
        let escaped = regex::escape(symbol);
        let re = Regex::new(&format!(r"\b{escaped}\b"))
            .map_err(|e| Error::invalid_argument(format!("invalid symbol regex: {e}")))?;
        let count = file_contents
            .iter()
            .map(|content| re.find_iter(content).count())
            .sum();
        map.insert(symbol.clone(), count);
    }
    Ok(map)
}
