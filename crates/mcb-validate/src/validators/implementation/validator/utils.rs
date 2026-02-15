use regex::Regex;

use crate::Result;

/// Iterate source lines, skipping comments and `#[cfg(test)]` modules.
/// Yields `(1-based line number, trimmed line)`.
pub fn source_lines(content: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut in_test_module = false;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        result.push((idx + 1, trimmed));
    }
    result
}

/// Filter out lines that belong to `#[cfg(test)]` regions.
/// Returns `(original 0-based index, trimmed line)` pairs.
pub fn non_test_lines<'a>(lines: &[&'a str]) -> Vec<(usize, &'a str)> {
    let mut result = Vec::new();
    let mut in_test_module = false;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("#[cfg(test)]") {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        result.push((i, trimmed));
    }
    result
}

/// Track function name from a regex pattern match.
pub fn track_fn_name(fn_pattern: Option<&Regex>, trimmed: &str, name: &mut String) {
    if let Some(re) = fn_pattern
        && let Some(cap) = re.captures(trimmed)
    {
        *name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }
}

/// Compile `(pattern_id, description)` pairs into `(Regex, &str)`.
pub fn compile_pattern_pairs<'a>(
    ids: &[(&str, &'a str)],
) -> Result<Vec<(&'static Regex, &'a str)>> {
    ids.iter()
        .map(|(id, desc)| required_pattern(id).map(|r| (r, *desc)))
        .collect()
}

pub fn required_patterns<'a>(ids: impl Iterator<Item = &'a str>) -> Result<Vec<&'static Regex>> {
    ids.map(required_pattern).collect()
}

pub(crate) use crate::pattern_registry::required_pattern;

/// Check if a line is a function signature or standalone brace.
pub fn is_fn_signature_or_brace(line: &str) -> bool {
    line.starts_with("fn ")
        || line.starts_with("pub fn ")
        || line.starts_with("async fn ")
        || line.starts_with("pub async fn ")
        || line == "{"
        || line == "}"
}

/// A parsed function with its body and metadata.
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub body_lines: Vec<String>,
    pub meaningful_body: Vec<String>,
    pub has_control_flow: bool,
}

/// Extract function bodies from non-test source lines.
/// Returns structured function info for each detected function.
pub fn extract_functions(fn_pattern: Option<&Regex>, lines: &[(usize, &str)]) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let (orig_idx, trimmed) = lines[i];
        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            let fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let fn_start = orig_idx + 1; // 1-based

            // Find function body extent by tracking braces
            let mut brace_depth: i32 = 0;
            let mut fn_started = false;
            let mut fn_end_idx = i;

            for (j, (_, line_content)) in lines[i..].iter().enumerate() {
                let opens = i32::try_from(line_content.chars().filter(|c| *c == '{').count())
                    .unwrap_or(i32::MAX);
                let closes = i32::try_from(line_content.chars().filter(|c| *c == '}').count())
                    .unwrap_or(i32::MAX);

                if opens > 0 {
                    fn_started = true;
                }
                brace_depth += opens - closes;
                if fn_started && brace_depth <= 0 {
                    fn_end_idx = i + j;
                    break;
                }
            }

            let body: Vec<String> = lines[i..=fn_end_idx]
                .iter()
                .map(|(_, l)| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with("//"))
                .collect();

            let meaningful = meaningful_lines(&body);
            let has_cf = has_control_flow(&body);

            functions.push(FunctionInfo {
                name: fn_name,
                start_line: fn_start,
                body_lines: body,
                meaningful_body: meaningful,
                has_control_flow: has_cf,
            });

            i = fn_end_idx;
        }
        i += 1;
    }
    functions
}

/// Extract functions with full body tracking, optionally tracking impl blocks.
pub fn extract_functions_with_body(
    fn_pattern: Option<&Regex>,
    impl_pattern: Option<&Regex>,
    lines: &[(usize, &str)],
    current_struct: &mut String,
) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();
    let mut current_fn_name = String::new();
    let mut fn_start_line: usize = 0;
    let mut fn_body_lines: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_fn = false;

    for &(orig_idx, trimmed) in lines {
        if trimmed.starts_with("//") {
            continue;
        }

        if let Some(re) = impl_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            *current_struct = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        }

        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            current_fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            fn_start_line = orig_idx + 1; // 1-based
            fn_body_lines.clear();
            in_fn = true;
            brace_depth = 0;
        }

        if in_fn {
            let opens =
                i32::try_from(trimmed.chars().filter(|c| *c == '{').count()).unwrap_or(i32::MAX);
            let closes =
                i32::try_from(trimmed.chars().filter(|c| *c == '}').count()).unwrap_or(i32::MAX);
            brace_depth += opens - closes;

            if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                fn_body_lines.push(trimmed.to_string());
            }

            if brace_depth <= 0 && opens > 0 {
                let meaningful = meaningful_lines(&fn_body_lines);
                functions.push(FunctionInfo {
                    name: current_fn_name.clone(),
                    start_line: fn_start_line,
                    body_lines: fn_body_lines.clone(),
                    meaningful_body: meaningful,
                    has_control_flow: has_control_flow(&fn_body_lines),
                });
                in_fn = false;
                fn_body_lines.clear();
            }
        }
    }
    functions
}

/// Filter a list of body lines to only meaningful ones (no braces, no `fn` sigs).
fn meaningful_lines(body: &[String]) -> Vec<String> {
    body.iter()
        .filter(|l| {
            !l.starts_with('{')
                && !l.starts_with('}')
                && *l != "{"
                && *l != "}"
                && !l.starts_with("fn ")
        })
        .cloned()
        .collect()
}

/// Check if any line in a function body contains control-flow keywords.
fn has_control_flow(body: &[String]) -> bool {
    body.iter().any(|line| {
        line.contains(" if ")
            || line.starts_with("if ")
            || line.contains("} else")
            || line.starts_with("match ")
            || line.contains(" match ")
            || line.starts_with("for ")
            || line.starts_with("while ")
            || line.starts_with("loop ")
            || line.contains(" else {")
            || line.contains("else {")
    })
}
