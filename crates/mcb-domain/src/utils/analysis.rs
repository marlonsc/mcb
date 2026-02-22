//!
//! **Documentation**: [docs/modules/domain.md#domain-utilities-utils](../../../../docs/modules/domain.md#domain-utilities-utils)
//!
use std::collections::HashMap;
use std::path::PathBuf;

use regex::Regex;

use crate::error::{Error, Result};
use crate::ports::{ComplexityFinding, DeadCodeFinding, TdgFinding};

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
///
/// # Errors
///
/// Returns an error if the internal complexity regex fails to compile.
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
#[must_use]
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

/// Extracts a code block defined by balanced braces `{}` from an iterator of lines.
/// Returns the number of lines from the start of the iterator that contain the balanced block.
#[must_use]
pub fn count_balanced_block_lines<I, S>(lines: I, max_search_offset: usize) -> Option<usize>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut brace_balance = 0;
    let mut found_start = false;

    for (offset, line) in lines.into_iter().enumerate() {
        let line_ref = line.as_ref();

        let open_count = line_ref.chars().filter(|c| *c == '{').count() as i32;
        let close_count = line_ref.chars().filter(|c| *c == '}').count() as i32;

        if open_count > 0 && !found_start {
            found_start = true;
        }

        if found_start {
            brace_balance += open_count;
            brace_balance -= close_count;

            if brace_balance <= 0 {
                return Some(offset + 1); // Length in lines
            }
        } else if offset > max_search_offset {
            return None;
        }
    }

    None
}

/// Checks if a symbol should be exempt from dead code detection (e.g. main, tests).
#[must_use]
pub fn is_exempt_symbol(name: &str) -> bool {
    matches!(name, "main") || name.starts_with("test_")
}

/// Counts occurrences of symbols across a set of file contents.
///
/// Returns a map of symbol -> count.
///
/// # Errors
///
/// Returns an error if a symbol produces an invalid regex pattern.
pub fn count_symbol_occurrences(
    file_contents: &[String],
    symbols: &[String],
) -> Result<HashMap<String, usize>> {
    let mut map = HashMap::new();
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

/// Collects function definitions from a set of source files.
///
/// Discovers all `fn` declarations (including `pub` and `async` variants),
/// computes their line number and cyclomatic complexity.
///
/// # Errors
///
/// Returns an error if the function-matching regex or complexity computation fails.
pub fn collect_functions(files: &[(PathBuf, String)]) -> Result<Vec<FunctionRecord>> {
    let fn_re = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)")
        .map_err(|e| Error::invalid_argument(format!("invalid function regex: {e}")))?;

    let mut records = Vec::new();
    for (file, content) in files {
        for captures in fn_re.captures_iter(content) {
            let Some(name_match) = captures.get(1) else {
                continue;
            };
            let name = name_match.as_str().to_owned();
            let fn_start = name_match.start();
            let line = content[..fn_start].bytes().filter(|b| *b == b'\n').count() + 1;
            let complexity = compute_complexity_score(content, fn_start)?;
            records.push(FunctionRecord {
                file: file.clone(),
                name,
                line,
                complexity,
            });
        }
    }
    Ok(records)
}

/// Filters functions exceeding a complexity threshold.
///
/// Returns a [`ComplexityFinding`] for each function whose score is above the threshold.
#[must_use]
pub fn filter_complex_functions(
    functions: Vec<FunctionRecord>,
    threshold: u32,
) -> Vec<ComplexityFinding> {
    functions
        .into_iter()
        .filter(|f| f.complexity > threshold)
        .map(|f| ComplexityFinding {
            file: f.file,
            function: f.name,
            complexity: f.complexity,
        })
        .collect()
}

/// Detects potentially dead functions by counting symbol occurrences.
///
/// A function is considered dead if it appears at most once across all file contents
/// and is not an exempt symbol (e.g. `main`, `test_*`).
///
/// # Errors
///
/// Returns an error if symbol occurrence counting fails.
pub fn detect_dead_functions(
    functions: Vec<FunctionRecord>,
    file_contents: &[String],
) -> Result<Vec<DeadCodeFinding>> {
    let names: Vec<String> = functions.iter().map(|f| f.name.clone()).collect();
    let occurrences = count_symbol_occurrences(file_contents, &names)?;

    Ok(functions
        .into_iter()
        .filter(|f| !is_exempt_symbol(&f.name))
        .filter(|f| occurrences.get(&f.name).copied().unwrap_or(0) <= 1)
        .map(|f| DeadCodeFinding {
            file: f.file,
            line: f.line,
            item_type: "function".to_owned(),
            name: f.name,
        })
        .collect())
}

/// Computes TDG (Technical Debt Grade) scores per file.
///
/// Aggregates max complexity, dead code count, and SLOC into a composite score.
/// Only files exceeding the threshold are returned.
#[must_use]
pub fn compute_tdg_scores(
    files: &[(PathBuf, String)],
    functions: Vec<FunctionRecord>,
    dead_code: &[DeadCodeFinding],
    threshold: u32,
) -> Vec<TdgFinding> {
    let mut complexity_by_file: HashMap<PathBuf, u32> = HashMap::new();
    for function in functions {
        let entry = complexity_by_file.entry(function.file).or_insert(0);
        *entry = (*entry).max(function.complexity);
    }

    let mut dead_by_file: HashMap<PathBuf, u32> = HashMap::new();
    for finding in dead_code {
        *dead_by_file.entry(finding.file.clone()).or_insert(0) += 1;
    }

    let mut findings = Vec::new();
    for (path, content) in files {
        let sloc = content.lines().filter(|l| !l.trim().is_empty()).count() as u32;
        let complexity = complexity_by_file.get(path).copied().unwrap_or(1);
        let dead = dead_by_file.get(path).copied().unwrap_or(0);

        let tdg_score = complexity.saturating_mul(2)
            + dead.saturating_mul(10)
            + ((sloc / 200).saturating_mul(10));

        if tdg_score > threshold {
            findings.push(TdgFinding {
                file: path.clone(),
                score: tdg_score,
            });
        }
    }

    findings
}
