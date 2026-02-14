use regex::Regex;

use crate::scan::for_each_scan_rs_path;
use crate::thresholds::thresholds;
use crate::{Result, Severity};

use super::{KissValidator, KissViolation};

impl KissValidator {
    pub fn validate_struct_fields(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let struct_pattern = match Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)\s*\{") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;
            let mut brace_depth: i32 = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                let open_c = line.chars().filter(|c| *c == '{').count();
                let close_c = line.chars().filter(|c| *c == '}').count();
                brace_depth += i32::try_from(open_c).unwrap_or(i32::MAX);
                brace_depth -= i32::try_from(close_c).unwrap_or(i32::MAX);

                if in_test_module && brace_depth < test_brace_depth {
                    in_test_module = false;
                }
                if in_test_module {
                    continue;
                }

                if let Some(cap) = struct_pattern.captures(line) {
                    let struct_name = cap.get(1).map_or("", |m| m.as_str());
                    let is_di_container = struct_name.ends_with("Context")
                        || struct_name.ends_with("Container")
                        || struct_name.ends_with("Components")
                        || struct_name.contains("Config")
                        || struct_name.contains("Settings")
                        || struct_name.ends_with("State");

                    let max_fields = if is_di_container {
                        thresholds().max_di_container_fields
                    } else {
                        self.max_struct_fields
                    };

                    let field_count = self.count_struct_fields(&lines, line_num);

                    if field_count > max_fields {
                        violations.push(KissViolation::StructTooManyFields {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            struct_name: struct_name.to_string(),
                            field_count,
                            max_allowed: max_fields,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    pub fn validate_function_params(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = match Regex::new(
            r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)",
        ) {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) {
                return Ok(());
            }

            let path_str = path.to_string_lossy();
            if path_str.ends_with("/admin/api.rs") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;
            let mut brace_depth: i32 = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                let open_c = line.chars().filter(|c| *c == '{').count();
                let close_c = line.chars().filter(|c| *c == '}').count();
                brace_depth += i32::try_from(open_c).unwrap_or(i32::MAX);
                brace_depth -= i32::try_from(close_c).unwrap_or(i32::MAX);

                if in_test_module && brace_depth < test_brace_depth {
                    in_test_module = false;
                }
                if in_test_module {
                    continue;
                }

                if !line.contains("fn ") {
                    continue;
                }

                let mut full_line = line.to_string();
                let mut idx = line_num + 1;
                while !full_line.contains(')') && idx < lines.len() {
                    full_line.push_str(lines[idx]);
                    idx += 1;
                }

                if let Some(cap) = fn_pattern.captures(&full_line) {
                    let fn_name = cap.get(1).map_or("", |m| m.as_str());
                    let params = cap.get(2).map_or("", |m| m.as_str());
                    let param_count = self.count_function_params(params);

                    if param_count > self.max_function_params {
                        violations.push(KissViolation::FunctionTooManyParams {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            function_name: fn_name.to_string(),
                            param_count,
                            max_allowed: self.max_function_params,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    pub fn validate_builder_complexity(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let builder_pattern =
            match Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Builder)\s*") {
                Ok(regex) => regex,
                Err(_) => return Ok(violations),
            };
        let option_pattern = match Regex::new(r"Option<") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = builder_pattern.captures(line) {
                    let builder_name = cap.get(1).map_or("", |m| m.as_str());
                    let optional_count =
                        self.count_optional_fields(&lines, line_num, &option_pattern);

                    if optional_count > self.max_builder_fields {
                        violations.push(KissViolation::BuilderTooComplex {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            builder_name: builder_name.to_string(),
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

    pub fn validate_nesting_depth(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let control_flow_pattern = match Regex::new(r"\b(if|match|for|while|loop)\b") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;

            let mut nesting_depth: usize = 0;
            let mut brace_depth: i32 = 0;
            let mut reported_lines: std::collections::HashSet<usize> =
                std::collections::HashSet::new();

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                if trimmed.starts_with("//") {
                    continue;
                }

                if control_flow_pattern.is_match(line) && line.contains('{') {
                    nesting_depth += 1;

                    if nesting_depth > self.max_nesting_depth {
                        let nearby_reported =
                            reported_lines.iter().any(|&l| l.abs_diff(line_num) < 5);

                        if !nearby_reported {
                            violations.push(KissViolation::DeepNesting {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                nesting_level: nesting_depth,
                                max_allowed: self.max_nesting_depth,
                                context: trimmed.chars().take(60).collect(),
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

    pub fn validate_function_length(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = match Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)") {
            Ok(regex) => regex,
            Err(_) => return Ok(violations),
        };

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) {
                return Ok(());
            }

            let path_str = path.to_string_lossy();
            if path_str.ends_with("/di/bootstrap.rs")
                || path_str.ends_with("/di/catalog.rs")
                || path_str.ends_with("/di/resolver.rs")
            {
                return Ok(());
            }

            if path_str.ends_with("/health.rs") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;
            let mut brace_depth: i32 = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                let open_c = line.chars().filter(|c| *c == '{').count();
                let close_c = line.chars().filter(|c| *c == '}').count();
                brace_depth += i32::try_from(open_c).unwrap_or(i32::MAX);
                brace_depth -= i32::try_from(close_c).unwrap_or(i32::MAX);

                if in_test_module && brace_depth < test_brace_depth {
                    in_test_module = false;
                }
                if in_test_module {
                    continue;
                }

                if let Some(cap) = fn_pattern.captures(line) {
                    let fn_name = cap.get(1).map_or("", |m| m.as_str());

                    if fn_name.starts_with("test_") {
                        continue;
                    }

                    if self.is_trait_fn_declaration(&lines, line_num) {
                        continue;
                    }

                    let line_count = self.count_function_lines(&lines, line_num);

                    if line_count > self.max_function_lines {
                        violations.push(KissViolation::FunctionTooLong {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            function_name: fn_name.to_string(),
                            line_count,
                            max_allowed: self.max_function_lines,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }
}
