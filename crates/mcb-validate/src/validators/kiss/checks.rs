//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use mcb_utils::constants::validate::{
    CFG_TEST_MARKER, COMMENT_PREFIX, SHORT_PREVIEW_LENGTH, TEST_FUNCTION_PREFIX,
};
use std::path::PathBuf;

use rust_code_analysis::SpaceKind;

use crate::ast::rca_helpers;
use crate::scan::for_each_scan_file;
use crate::thresholds::thresholds;
use crate::{Result, Severity};
use mcb_utils::utils::regex::compile_regex;

use super::{KissValidator, KissViolation};
use mcb_utils::constants::validate::{
    DI_CONTAINER_CONTAINS, DI_CONTAINER_SUFFIXES, NESTING_PROXIMITY_THRESHOLD,
};

/// Per-file state for nesting-depth analysis.
#[derive(Default)]
struct NestingState {
    in_test_module: bool,
    test_brace_depth: i32,
    nesting_depth: usize,
    brace_depth: i32,
    reported_lines: std::collections::HashSet<usize>,
}

impl NestingState {
    /// Whether a `DeepNesting` violation was already reported close enough to
    /// `line_num` to suppress a duplicate.
    fn has_nearby_report(&self, line_num: usize) -> bool {
        self.reported_lines
            .iter()
            .any(|&l| l.abs_diff(line_num) < NESTING_PROXIMITY_THRESHOLD)
    }
}

impl KissValidator {
    fn update_test_module_tracking(
        trimmed: &str,
        line: &str,
        in_test_module: &mut bool,
        test_brace_depth: &mut i32,
        brace_depth: &mut i32,
    ) {
        if trimmed.contains(CFG_TEST_MARKER) {
            *in_test_module = true;
            *test_brace_depth = *brace_depth;
        }

        let open_c = line.chars().filter(|c| *c == '{').count();
        let close_c = line.chars().filter(|c| *c == '}').count();
        *brace_depth += i32::try_from(open_c).unwrap_or(i32::MAX);
        *brace_depth -= i32::try_from(close_c).unwrap_or(i32::MAX);

        if *in_test_module && *brace_depth < *test_brace_depth {
            *in_test_module = false;
        }
    }

    fn for_each_kiss_file<F>(&self, skip_file_checks: bool, mut f: F) -> Result<()>
    where
        F: FnMut(PathBuf, String) -> Result<()>,
    {
        for_each_scan_file(
            &self.config,
            Some(LanguageId::Rust),
            false,
            |entry, src_dir| {
                let path = &entry.absolute_path;
                if self.should_skip_crate(src_dir)
                    || (skip_file_checks && self.should_skip_file(path))
                {
                    return Ok(());
                }

                let ctx = ValidationRunContext::active_or_build(&self.config)?;
                let cached = ctx
                    .read_cached(path)
                    .map_err(|e| crate::ValidationError::Config(e.to_string()))?;
                let content = cached.to_string();
                f(path.clone(), content)
            },
        )
    }

    /// Helper for validating RCA-based function metrics.
    /// Handles the boilerplate: `for_each_kiss_file` → `parse_file_spaces` → iterate functions → check metric.
    /// The closure `check` is called for each non-test function and should return Some(violation) if the metric exceeds threshold.
    fn validate_rca_fn_metric<F>(
        &self,
        check_label: &str,
        mut check: F,
    ) -> Result<Vec<KissViolation>>
    where
        F: FnMut(&rust_code_analysis::FuncSpace, &PathBuf) -> Option<KissViolation>,
    {
        let mut violations = Vec::new();
        mcb_domain::debug!("kiss", &format!("Checking {check_label}"));

        self.for_each_kiss_file(true, |path, content| {
            let Some(root) = rca_helpers::parse_file_spaces(&path, &content) else {
                return Ok(());
            };
            mcb_domain::trace!("kiss", check_label, &format!("file={}", path.display()));

            for space in rca_helpers::collect_spaces_of_kind(&root, &content, SpaceKind::Function) {
                let fn_name = space.name.as_deref().unwrap_or("");
                if fn_name.starts_with(TEST_FUNCTION_PREFIX) {
                    continue;
                }
                if let Some(violation) = check(space, &path) {
                    violations.push(violation);
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    fn for_each_non_test_line<F>(lines: &[&str], mut f: F)
    where
        F: FnMut(usize, &str, &str),
    {
        let mut in_test_module = false;
        let mut test_brace_depth: i32 = 0;
        let mut brace_depth: i32 = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            Self::update_test_module_tracking(
                trimmed,
                line,
                &mut in_test_module,
                &mut test_brace_depth,
                &mut brace_depth,
            );
            if in_test_module {
                continue;
            }

            f(line_num, line, trimmed);
        }
    }

    /// Detects structs with too many fields, excluding DI containers and config types.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_struct_fields(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let struct_pattern = compile_regex(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)\s*\{")?;

        self.for_each_kiss_file(false, |path, content| {
            let lines: Vec<&str> = content.lines().collect();
            Self::for_each_non_test_line(&lines, |line_num, line, _trimmed| {
                if let Some(violation) =
                    self.struct_field_violation(&path, &lines, line, line_num, &struct_pattern)
                {
                    violations.push(violation);
                }
            });
            Ok(())
        })?;

        Ok(violations)
    }

    /// Returns a `StructTooManyFields` violation if `line` opens a struct whose
    /// field count exceeds the limit (relaxed for DI containers), else `None`.
    fn struct_field_violation(
        &self,
        path: &std::path::Path,
        lines: &[&str],
        line: &str,
        line_num: usize,
        struct_pattern: &regex::Regex,
    ) -> Option<KissViolation> {
        let cap = struct_pattern.captures(line)?;
        let struct_name = cap.get(1).map_or("", |m| m.as_str());
        let is_di_container = DI_CONTAINER_SUFFIXES
            .iter()
            .any(|s| struct_name.ends_with(s))
            || DI_CONTAINER_CONTAINS
                .iter()
                .any(|s| struct_name.contains(s));

        let max_fields = if is_di_container {
            thresholds().max_di_container_fields
        } else {
            self.max_struct_fields
        };
        let field_count = Self::count_struct_fields(lines, line_num);

        (field_count > max_fields).then(|| KissViolation::StructTooManyFields {
            file: path.to_path_buf(),
            line: line_num + 1,
            struct_name: struct_name.to_owned(),
            field_count,
            max_allowed: max_fields,
            severity: Severity::Warning,
        })
    }

    /// Detects functions with too many parameters using rust-code-analysis.
    ///
    /// Uses RCA's `get_function_spaces()` to obtain per-function `nargs` metrics
    /// instead of regex-based parameter counting.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_function_params(&self) -> Result<Vec<KissViolation>> {
        self.validate_rca_fn_metric("function parameter counts", |space, path| {
            let param_count = space.metrics.nargs.fn_args().round() as usize;
            if param_count > self.max_function_params {
                Some(KissViolation::FunctionTooManyParams {
                    file: path.clone(),
                    line: space.start_line,
                    function_name: space.name.as_deref().unwrap_or("").to_owned(),
                    param_count,
                    max_allowed: self.max_function_params,
                    severity: Severity::Warning,
                })
            } else {
                None
            }
        })
    }

    /// Detects builder structs with too many optional fields.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_builder_complexity(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let builder_pattern = compile_regex(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Builder)\s*")?;
        let option_pattern = compile_regex("Option<")?;

        self.for_each_kiss_file(false, |path, content| {
            let lines: Vec<&str> = content.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = builder_pattern.captures(line) {
                    let builder_name = cap.get(1).map_or("", |m| m.as_str());
                    let optional_count =
                        Self::count_optional_fields(&lines, line_num, &option_pattern);

                    if optional_count > self.max_builder_fields {
                        violations.push(KissViolation::BuilderTooComplex {
                            file: path.clone(),
                            line: line_num + 1,
                            builder_name: builder_name.to_owned(),
                            optional_field_count: optional_count,
                            max_allowed: self.max_builder_fields,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Detects code blocks with excessive nesting depth.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_nesting_depth(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let control_flow_pattern = compile_regex(r"\b(if|match|for|while|loop)\b")?;

        self.for_each_kiss_file(false, |path, content| {
            let lines: Vec<&str> = content.lines().collect();
            let mut state = NestingState::default();
            for (line_num, line) in lines.iter().enumerate() {
                self.process_nesting_line(
                    &path,
                    line,
                    line_num,
                    &control_flow_pattern,
                    &mut state,
                    &mut violations,
                );
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Update nesting/brace tracking for one line and report a `DeepNesting`
    /// violation when the control-flow nesting exceeds the configured maximum.
    fn process_nesting_line(
        &self,
        path: &std::path::Path,
        line: &str,
        line_num: usize,
        control_flow_pattern: &regex::Regex,
        state: &mut NestingState,
        violations: &mut Vec<KissViolation>,
    ) {
        let trimmed = line.trim();

        if trimmed.contains(CFG_TEST_MARKER) {
            state.in_test_module = true;
            state.test_brace_depth = state.brace_depth;
        }

        if trimmed.starts_with(COMMENT_PREFIX) {
            return;
        }

        if control_flow_pattern.is_match(line) && line.contains('{') {
            state.nesting_depth += 1;
            if state.nesting_depth > self.max_nesting_depth && !state.has_nearby_report(line_num) {
                violations.push(KissViolation::DeepNesting {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    nesting_level: state.nesting_depth,
                    max_allowed: self.max_nesting_depth,
                    context: trimmed.chars().take(SHORT_PREVIEW_LENGTH).collect(),
                    severity: Severity::Warning,
                });
                state.reported_lines.insert(line_num);
            }
        }

        let open_braces = line.chars().filter(|c| *c == '{').count();
        let close_braces = line.chars().filter(|c| *c == '}').count();
        state.brace_depth += i32::try_from(open_braces).unwrap_or(i32::MAX);
        state.brace_depth -= i32::try_from(close_braces).unwrap_or(i32::MAX);

        if close_braces > 0 && state.nesting_depth > 0 {
            state.nesting_depth = state.nesting_depth.saturating_sub(close_braces);
        }

        if state.in_test_module && state.brace_depth < state.test_brace_depth {
            state.in_test_module = false;
        }
    }

    /// Detects functions that exceed the maximum allowed line count using rust-code-analysis.
    ///
    /// Uses RCA's `get_function_spaces()` to obtain per-function SLOC metrics
    /// instead of regex + brace counting.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_function_length(&self) -> Result<Vec<KissViolation>> {
        self.validate_rca_fn_metric("function lengths", |space, path| {
            let line_count = space.metrics.loc.sloc().round() as usize;
            if line_count > self.max_function_lines {
                Some(KissViolation::FunctionTooLong {
                    file: path.clone(),
                    line: space.start_line,
                    function_name: space.name.as_deref().unwrap_or("").to_owned(),
                    line_count,
                    max_allowed: self.max_function_lines,
                    severity: Severity::Warning,
                })
            } else {
                None
            }
        })
    }
}
