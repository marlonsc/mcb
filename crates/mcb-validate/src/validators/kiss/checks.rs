use crate::constants::common::{
    CFG_TEST_MARKER, COMMENT_PREFIX, FN_PREFIX, SHORT_PREVIEW_LENGTH, TEST_FUNCTION_PREFIX,
};
use crate::filters::LanguageId;
use std::path::PathBuf;

use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::thresholds::thresholds;
use crate::{Result, Severity};

use super::constants::{DI_CONTAINER_CONTAINS, DI_CONTAINER_SUFFIXES, NESTING_PROXIMITY_THRESHOLD};
use super::{KissValidator, KissViolation};

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

                let content = std::fs::read_to_string(path)?;
                f(path.clone(), content)
            },
        )
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
        let struct_pattern = match compile_regex(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)\s*\{") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        self.for_each_kiss_file(false, |path, content| {
            let lines: Vec<&str> = content.lines().collect();
            Self::for_each_non_test_line(&lines, |line_num, line, _trimmed| {
                if let Some(cap) = struct_pattern.captures(line) {
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

                    let field_count = Self::count_struct_fields(&lines, line_num);

                    if field_count > max_fields {
                        violations.push(KissViolation::StructTooManyFields {
                            file: path.clone(),
                            line: line_num + 1,
                            struct_name: struct_name.to_owned(),
                            field_count,
                            max_allowed: max_fields,
                            severity: Severity::Warning,
                        });
                    }
                }
            });
            Ok(())
        })?;

        Ok(violations)
    }

    /// Detects functions with too many parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_function_params(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = match compile_regex(
            r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)",
        ) {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        self.for_each_kiss_file(true, |path, content| {
            let lines: Vec<&str> = content.lines().collect();
            Self::for_each_non_test_line(&lines, |line_num, line, _trimmed| {
                if !line.contains(FN_PREFIX) {
                    return;
                }

                let mut full_line = line.to_owned();
                let mut idx = line_num + 1;
                while !full_line.contains(')') && idx < lines.len() {
                    full_line.push_str(lines[idx]);
                    idx += 1;
                }

                if let Some(cap) = fn_pattern.captures(&full_line) {
                    let fn_name = cap.get(1).map_or("", |m| m.as_str());
                    let params = cap.get(2).map_or("", |m| m.as_str());
                    let param_count = Self::count_function_params(params);

                    if param_count > self.max_function_params {
                        violations.push(KissViolation::FunctionTooManyParams {
                            file: path.clone(),
                            line: line_num + 1,
                            function_name: fn_name.to_owned(),
                            param_count,
                            max_allowed: self.max_function_params,
                            severity: Severity::Warning,
                        });
                    }
                }
            });
            Ok(())
        })?;

        Ok(violations)
    }

    /// Detects builder structs with too many optional fields.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_builder_complexity(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let builder_pattern =
            match compile_regex(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Builder)\s*") {
                Ok(regex) => regex,
                Err(_) => return Ok(violations),
            };
        let option_pattern = match compile_regex("Option<") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

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
        let control_flow_pattern = match compile_regex(r"\b(if|match|for|while|loop)\b") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        self.for_each_kiss_file(false, |path, content| {
            let lines: Vec<&str> = content.lines().collect();

            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;

            let mut nesting_depth: usize = 0;
            let mut brace_depth: i32 = 0;
            let mut reported_lines: std::collections::HashSet<usize> =
                std::collections::HashSet::new();

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains(CFG_TEST_MARKER) {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                if trimmed.starts_with(COMMENT_PREFIX) {
                    continue;
                }

                if control_flow_pattern.is_match(line) && line.contains('{') {
                    nesting_depth += 1;

                    if nesting_depth > self.max_nesting_depth {
                        let nearby_reported = reported_lines
                            .iter()
                            .any(|&l| l.abs_diff(line_num) < NESTING_PROXIMITY_THRESHOLD);

                        if !nearby_reported {
                            violations.push(KissViolation::DeepNesting {
                                file: path.clone(),
                                line: line_num + 1,
                                nesting_level: nesting_depth,
                                max_allowed: self.max_nesting_depth,
                                context: trimmed.chars().take(SHORT_PREVIEW_LENGTH).collect(),
                                severity: Severity::Warning,
                            });
                            reported_lines.insert(line_num);
                        }
                    }
                }

                let open_braces = line.chars().filter(|c| *c == '{').count();
                let close_braces = line.chars().filter(|c| *c == '}').count();
                brace_depth += i32::try_from(open_braces).unwrap_or(i32::MAX);
                brace_depth -= i32::try_from(close_braces).unwrap_or(i32::MAX);

                if close_braces > 0 && nesting_depth > 0 {
                    nesting_depth = nesting_depth.saturating_sub(close_braces);
                }

                if in_test_module && brace_depth < test_brace_depth {
                    in_test_module = false;
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Detects functions that exceed the maximum allowed line count.
    ///
    /// # Errors
    ///
    /// Returns an error if file scanning or reading fails.
    pub fn validate_function_length(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = match compile_regex(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        self.for_each_kiss_file(true, |path, content| {
            let lines: Vec<&str> = content.lines().collect();
            Self::for_each_non_test_line(&lines, |line_num, line, _trimmed| {
                if let Some(cap) = fn_pattern.captures(line) {
                    let fn_name = cap.get(1).map_or("", |m| m.as_str());

                    if fn_name.starts_with(TEST_FUNCTION_PREFIX) {
                        return;
                    }

                    if Self::is_trait_fn_declaration(&lines, line_num) {
                        return;
                    }

                    let line_count = Self::count_function_lines(&lines, line_num);

                    if line_count > self.max_function_lines {
                        violations.push(KissViolation::FunctionTooLong {
                            file: path.clone(),
                            line: line_num + 1,
                            function_name: fn_name.to_owned(),
                            line_count,
                            max_allowed: self.max_function_lines,
                            severity: Severity::Warning,
                        });
                    }
                }
            });
            Ok(())
        })?;

        Ok(violations)
    }
}
