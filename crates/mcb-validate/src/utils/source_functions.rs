use regex::Regex;

use crate::constants::common::{COMMENT_PREFIX, FN_PREFIX};

use super::FunctionInfo;

pub(super) fn extract_functions_impl(
    fn_pattern: Option<&Regex>,
    lines: &[(usize, &str)],
) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let (orig_idx, trimmed) = lines[i];
        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            let fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
            let fn_start = orig_idx + 1;

            let fn_end_idx = find_function_end(lines, i);
            let body: Vec<String> = lines[i..=fn_end_idx]
                .iter()
                .map(|(_, l)| l.trim().to_owned())
                .filter(|l| !l.is_empty() && !l.starts_with(COMMENT_PREFIX))
                .collect();

            functions.push(FunctionInfo {
                name: fn_name,
                start_line: fn_start,
                meaningful_body: meaningful_lines(&body),
                has_control_flow: has_control_flow(&body),
                body_lines: body,
            });

            i = fn_end_idx;
        }
        i += 1;
    }

    functions
}

pub(super) fn extract_functions_with_body_impl(
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
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        if let Some(re) = impl_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            *current_struct = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
        }

        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            current_fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
            fn_start_line = orig_idx + 1;
            fn_body_lines.clear();
            in_fn = true;
            brace_depth = 0;
        }

        if !in_fn {
            continue;
        }

        let opens =
            i32::try_from(trimmed.chars().filter(|c| *c == '{').count()).unwrap_or(i32::MAX);
        let closes =
            i32::try_from(trimmed.chars().filter(|c| *c == '}').count()).unwrap_or(i32::MAX);
        brace_depth += opens - closes;

        if !trimmed.is_empty() && !trimmed.starts_with("#[") {
            fn_body_lines.push(trimmed.to_owned());
        }

        if brace_depth <= 0 && opens > 0 {
            functions.push(FunctionInfo {
                name: current_fn_name.clone(),
                start_line: fn_start_line,
                meaningful_body: meaningful_lines(&fn_body_lines),
                has_control_flow: has_control_flow(&fn_body_lines),
                body_lines: fn_body_lines.clone(),
            });
            in_fn = false;
            fn_body_lines.clear();
        }
    }

    functions
}

fn find_function_end(lines: &[(usize, &str)], start_idx: usize) -> usize {
    let mut brace_depth: i32 = 0;
    let mut fn_started = false;
    let mut fn_end_idx = start_idx;

    for (j, (_, line_content)) in lines[start_idx..].iter().enumerate() {
        let opens =
            i32::try_from(line_content.chars().filter(|c| *c == '{').count()).unwrap_or(i32::MAX);
        let closes =
            i32::try_from(line_content.chars().filter(|c| *c == '}').count()).unwrap_or(i32::MAX);

        if opens > 0 {
            fn_started = true;
        }
        brace_depth += opens - closes;
        if fn_started && brace_depth <= 0 {
            fn_end_idx = start_idx + j;
            break;
        }
    }

    fn_end_idx
}

fn meaningful_lines(body: &[String]) -> Vec<String> {
    body.iter()
        .filter(|line| !is_structural_line(line) && !line.starts_with(FN_PREFIX))
        .cloned()
        .collect()
}

fn is_structural_line(line: &str) -> bool {
    line.starts_with('{') || line.starts_with('}') || matches!(line, "{" | "}")
}

fn has_control_flow(body: &[String]) -> bool {
    const CONTAINS_TOKENS: [&str; 4] = [" if ", "} else", " match ", " else {"];
    const STARTS_WITH_TOKENS: [&str; 5] = ["if ", "match ", "for ", "while ", "loop "];

    body.iter().any(|line| {
        line.contains("else {")
            || CONTAINS_TOKENS.iter().any(|token| line.contains(token))
            || STARTS_WITH_TOKENS
                .iter()
                .any(|token| line.starts_with(token))
    })
}
