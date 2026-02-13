//! Implementation validator implementation

use std::path::{Path, PathBuf};

use regex::Regex;

use super::violation::ImplementationViolation;
use crate::config::ImplementationRulesConfig;
use crate::pattern_registry::{required_pattern, required_patterns};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity, ValidationConfig};

/// Implementation quality validator
pub struct ImplementationQualityValidator {
    config: ValidationConfig,
    rules: ImplementationRulesConfig,
}

impl ImplementationQualityValidator {
    /// Create a new implementation quality validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let config = ValidationConfig::new(workspace_root);
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        Self { config, rules }
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig, rules: &ImplementationRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Run all implementation quality validations
    pub fn validate_all(&self) -> Result<Vec<ImplementationViolation>> {
        let mut all = Vec::new();
        all.extend(self.validate_empty_methods()?);
        all.extend(self.validate_hardcoded_returns()?);
        all.extend(self.validate_stub_macros()?);
        all.extend(self.validate_empty_catch_alls()?);
        all.extend(self.validate_pass_through_wrappers()?);
        all.extend(self.validate_log_only_methods()?);
        Ok(all)
    }

    // ── Shared helpers ────────────────────────────────────────────────

    /// Iterate over all non-test `.rs` source files in the configured scan dirs.
    fn for_each_source_file<F>(&self, mut visitor: F) -> Result<Vec<ImplementationViolation>>
    where
        F: FnMut(&Path, &str) -> Vec<ImplementationViolation>,
    {
        let mut violations = Vec::new();
        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) || is_test_path(path) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            violations.extend(visitor(path, &content));
            Ok(())
        })?;
        Ok(violations)
    }

    /// Like [`for_each_source_file`] but also skips null/fake provider files.
    fn for_each_prod_source_file<F>(&self, mut visitor: F) -> Result<Vec<ImplementationViolation>>
    where
        F: FnMut(&Path, &str) -> Vec<ImplementationViolation>,
    {
        self.for_each_source_file(|path, content| {
            let fname = file_name_str(path);
            if fname.contains("null") || fname.contains("fake") {
                return Vec::new();
            }
            visitor(path, content)
        })
    }

    // ── Validation methods ────────────────────────────────────────────

    /// Detect empty method bodies
    pub fn validate_empty_methods(&self) -> Result<Vec<ImplementationViolation>> {
        let empty_pattern_ids = [
            ("IMPL001.empty_ok_unit", "Ok(())"),
            ("IMPL001.empty_none", "None"),
            ("IMPL001.empty_vec_new", "Vec::new()"),
            ("IMPL001.empty_string_new", "String::new()"),
            ("IMPL001.empty_default", "Default::default()"),
            ("IMPL001.empty_ok_vec", "Ok(Vec::new())"),
            ("IMPL001.empty_ok_none", "Ok(None)"),
            ("IMPL001.empty_ok_false", "Ok(false)"),
            ("IMPL001.empty_ok_zero", "Ok(0)"),
        ];

        let fn_pattern = required_pattern("IMPL001.fn_decl")?;
        let compiled = compile_pattern_pairs(&empty_pattern_ids)?;

        self.for_each_prod_source_file(|entry, content| {
            let mut violations = Vec::new();
            let mut current_fn_name = String::new();
            for (line_num, trimmed) in source_lines(content) {
                track_fn_name(Some(fn_pattern), trimmed, &mut current_fn_name);
                for (pattern, desc) in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::EmptyMethodBody {
                            file: entry.to_path_buf(),
                            line: line_num,
                            method_name: current_fn_name.clone(),
                            pattern: desc.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            violations
        })
    }

    /// Detect hardcoded return values
    pub fn validate_hardcoded_returns(&self) -> Result<Vec<ImplementationViolation>> {
        let hardcoded_pattern_ids = [
            ("IMPL001.return_true", "true"),
            ("IMPL001.return_false", "false"),
            ("IMPL001.return_zero", "0"),
            ("IMPL001.return_one", "1"),
            ("IMPL001.return_empty_string", "empty string"),
            ("IMPL001.return_hardcoded_string", "hardcoded string"),
        ];

        let fn_pattern = required_pattern("IMPL001.fn_decl")?;
        let compiled = compile_pattern_pairs(&hardcoded_pattern_ids)?;

        self.for_each_prod_source_file(|entry, content| {
            let fname = file_name_str(entry);
            if fname == "constants.rs" {
                return Vec::new();
            }

            let mut violations = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            let non_test_lines = non_test_lines(&lines);

            for func in extract_functions(Some(fn_pattern), &non_test_lines) {
                if func.has_control_flow {
                    continue;
                }
                for line in &func.body_lines {
                    if is_fn_signature_or_brace(line) {
                        continue;
                    }
                    for (pattern, desc) in &compiled {
                        if pattern.is_match(line) {
                            violations.push(ImplementationViolation::HardcodedReturnValue {
                                file: entry.to_path_buf(),
                                line: func.start_line,
                                method_name: func.name.clone(),
                                return_value: desc.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
            violations
        })
    }

    /// Detect stub macros (todo!, unimplemented!)
    pub fn validate_stub_macros(&self) -> Result<Vec<ImplementationViolation>> {
        use crate::constants::STUB_PANIC_LABEL;
        let stub_pattern_ids = [
            ("IMPL001.stub_todo", "todo"),
            ("IMPL001.stub_unimplemented", "unimplemented"),
            ("IMPL001.stub_panic_not_impl", "panic(not implemented)"),
            ("IMPL001.stub_panic_todo", STUB_PANIC_LABEL),
        ];

        let fn_pattern = required_pattern("IMPL001.fn_decl")?;
        let compiled = compile_pattern_pairs(&stub_pattern_ids)?;

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            let mut current_fn_name = String::new();
            for (line_num, trimmed) in source_lines(content) {
                track_fn_name(Some(fn_pattern), trimmed, &mut current_fn_name);
                for (pattern, macro_type) in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::StubMacro {
                            file: entry.to_path_buf(),
                            line: line_num,
                            method_name: current_fn_name.clone(),
                            macro_type: macro_type.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            violations
        })
    }

    /// Detect empty catch-all match arms
    pub fn validate_empty_catch_alls(&self) -> Result<Vec<ImplementationViolation>> {
        let catchall_ids = [
            "IMPL001.catchall_empty",
            "IMPL001.catchall_unit",
            "IMPL001.catchall_ok_unit",
            "IMPL001.catchall_none",
            "IMPL001.catchall_continue",
        ];

        let compiled = required_patterns(catchall_ids.iter().copied())?;

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            for (line_num, trimmed) in source_lines(content) {
                for pattern in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::EmptyCatchAll {
                            file: entry.to_path_buf(),
                            line: line_num,
                            context: trimmed.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            violations
        })
    }

    /// Detect pass-through wrappers
    pub fn validate_pass_through_wrappers(&self) -> Result<Vec<ImplementationViolation>> {
        let passthrough_pattern = required_pattern("IMPL001.passthrough")?;
        let fn_pattern = required_pattern("IMPL001.fn_decl")?;
        let impl_pattern = required_pattern("IMPL001.impl_decl")?;

        self.for_each_source_file(|entry, content| {
            let fname = file_name_str(entry);
            if fname.contains("adapter") || fname.contains("wrapper") {
                return Vec::new();
            }

            let mut violations = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            let non_test = non_test_lines(&lines);

            // Track current impl block
            let mut current_struct_name = String::new();
            for func in extract_functions_with_body(
                Some(fn_pattern),
                Some(impl_pattern),
                &non_test,
                &mut current_struct_name,
            ) {
                if func.meaningful_body.len() != 1 {
                    continue;
                }
                if let Some(cap) = passthrough_pattern.captures(&func.meaningful_body[0]) {
                    let field = cap.get(1).map_or("", |m| m.as_str());
                    let method = cap.get(2).map_or("", |m| m.as_str());
                    if method == func.name || method.starts_with(&func.name) {
                        violations.push(ImplementationViolation::PassThroughWrapper {
                            file: entry.to_path_buf(),
                            line: func.start_line,
                            struct_name: current_struct_name.clone(),
                            method_name: func.name.clone(),
                            delegated_to: format!("self.{field}.{method}()"),
                            severity: Severity::Info,
                        });
                    }
                }
            }
            violations
        })
    }

    /// Detect log-only methods
    pub fn validate_log_only_methods(&self) -> Result<Vec<ImplementationViolation>> {
        let log_pattern_ids = [
            "IMPL001.log_tracing",
            "IMPL001.log_log",
            "IMPL001.log_println",
            "IMPL001.log_eprintln",
        ];

        let fn_pattern = required_pattern("IMPL001.fn_decl")?;
        let compiled_log = required_patterns(log_pattern_ids.iter().copied())?;

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            let non_test = non_test_lines(&lines);
            let mut dummy = String::new();

            for func in extract_functions_with_body(Some(fn_pattern), None, &non_test, &mut dummy) {
                if func.meaningful_body.is_empty() || func.meaningful_body.len() > 3 {
                    continue;
                }
                let all_logging = func
                    .meaningful_body
                    .iter()
                    .all(|line| compiled_log.iter().any(|p| p.is_match(line)));
                if all_logging {
                    violations.push(ImplementationViolation::LogOnlyMethod {
                        file: entry.to_path_buf(),
                        line: func.start_line,
                        method_name: func.name.clone(),
                        severity: Severity::Warning,
                    });
                }
            }
            violations
        })
    }

    /// Check if a crate should be skipped based on configuration
    fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let path_str = src_dir.to_string_lossy();
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}

impl_validator!(
    ImplementationQualityValidator,
    "implementation",
    "Validates implementation quality patterns (empty methods, hardcoded returns, stubs)"
);

// ── Free helper functions ─────────────────────────────────────────────

fn is_test_path(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.contains("/tests/")
}

fn file_name_str(path: &Path) -> &str {
    path.file_name().and_then(|n| n.to_str()).unwrap_or("")
}

/// Iterate source lines, skipping comments and `#[cfg(test)]` modules.
/// Yields `(1-based line number, trimmed line)`.
fn source_lines(content: &str) -> Vec<(usize, &str)> {
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
fn non_test_lines<'a>(lines: &[&'a str]) -> Vec<(usize, &'a str)> {
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
fn track_fn_name(fn_pattern: Option<&Regex>, trimmed: &str, name: &mut String) {
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
fn compile_pattern_pairs<'a>(ids: &[(&str, &'a str)]) -> Result<Vec<(&'static Regex, &'a str)>> {
    ids.iter()
        .map(|(id, desc)| required_pattern(id).map(|r| (r, *desc)))
        .collect()
}

/// Check if a line is a function signature or standalone brace.
fn is_fn_signature_or_brace(line: &str) -> bool {
    line.starts_with("fn ")
        || line.starts_with("pub fn ")
        || line.starts_with("async fn ")
        || line.starts_with("pub async fn ")
        || line == "{"
        || line == "}"
}

/// A parsed function with its body and metadata.
struct FunctionInfo {
    name: String,
    start_line: usize,
    body_lines: Vec<String>,
    meaningful_body: Vec<String>,
    has_control_flow: bool,
}

/// Extract function bodies from non-test source lines.
/// Returns structured function info for each detected function.
fn extract_functions(fn_pattern: Option<&Regex>, lines: &[(usize, &str)]) -> Vec<FunctionInfo> {
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
fn extract_functions_with_body(
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
