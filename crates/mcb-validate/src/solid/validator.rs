//! Solid validator implementation
//!
//! This module provides the `SolidValidator` which enforces SOLID design principles:
//! - Single Responsibility Principle (SRP)
//! - Open/Closed Principle (OCP)
//! - Liskov Substitution Principle (LSP)
//! - Interface Segregation Principle (ISP)
//! - Dependency Inversion Principle (DIP)

use std::path::PathBuf;

use regex::Regex;

use super::violation::SolidViolation;
use crate::pattern_registry::required_pattern;
use crate::scan::for_each_rs_under_root;
use crate::thresholds::thresholds;
use crate::{Result, Severity, ValidationConfig};

/// SOLID principles validator
pub struct SolidValidator {
    /// Configuration for validation scans
    config: ValidationConfig,
    /// Maximum number of methods allowed in a trait
    max_trait_methods: usize,
    /// Maximum number of lines allowed in a struct definition
    max_struct_lines: usize,
    /// Maximum number of arms allowed in a match expression
    max_match_arms: usize,
    /// Maximum number of methods allowed in an impl block
    max_impl_methods: usize,
}

impl SolidValidator {
    /// Create a new SOLID validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        let t = thresholds();
        Self {
            config,
            max_trait_methods: t.max_trait_methods,
            max_struct_lines: t.max_struct_lines,
            max_match_arms: t.max_match_arms,
            max_impl_methods: t.max_impl_methods,
        }
    }

    /// Run all SOLID validations
    ///
    /// # Errors
    /// Returns an error if file traversal or pattern compilation fails.
    pub fn validate_all(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_srp()?);
        violations.extend(self.validate_ocp()?);
        violations.extend(self.validate_isp()?);
        violations.extend(self.validate_lsp()?);
        violations.extend(self.validate_impl_method_count()?);
        Ok(violations)
    }

    /// Generic helper: iterate over all Rust files in crate source directories
    fn for_each_rust_file<F>(&self, mut visitor: F) -> Result<()>
    where
        F: FnMut(PathBuf, Vec<&str>) -> Result<()>,
    {
        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for_each_rs_under_root(&self.config, &src_dir, |path| {
                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();
                visitor(path.to_path_buf(), lines)
            })?;
        }
        Ok(())
    }

    /// Helper: Scan declaration blocks and count methods
    /// Helper: Scan declaration blocks and count methods
    fn scan_decl_blocks<F>(
        &self,
        decl_pattern: &Regex,
        member_fn_pattern: &Regex,
        count_fn: fn(&[&str], usize, &Regex) -> usize,
        max_allowed: usize,
        make_violation: F,
    ) -> Result<Vec<SolidViolation>>
    where
        F: Fn(PathBuf, usize, &str, usize, usize) -> SolidViolation,
    {
        let mut violations = Vec::new();

        self.for_each_rust_file(|path, lines| {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = decl_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    let method_count = count_fn(&lines, line_num, member_fn_pattern);

                    if method_count > max_allowed {
                        violations.push(make_violation(
                            path.clone(),
                            line_num + 1,
                            name,
                            method_count,
                            max_allowed,
                        ));
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// SRP: Check for structs/impls that are too large
    ///
    /// # Errors
    /// Returns an error if patterns fail to compile or file reading fails.
    pub fn validate_srp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_pattern = required_pattern("SOLID002.impl_decl")?;
        let struct_pattern = required_pattern("SOLID002.struct_decl")?;

        self.for_each_rust_file(|path, lines| {
            let mut structs_in_file: Vec<(String, usize)> = Vec::new();

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = struct_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    structs_in_file.push((name.to_string(), line_num + 1));
                }

                if let Some(cap) = impl_pattern.captures(line) {
                    let name = cap.get(1).or(cap.get(2)).map_or("", |m| m.as_str());
                    let block_lines = Self::count_block_lines(&lines, line_num);

                    if block_lines > self.max_struct_lines {
                        violations.push(SolidViolation::TooManyResponsibilities {
                            file: path.clone(),
                            line: line_num + 1,
                            item_type: "impl".to_string(),
                            item_name: name.to_string(),
                            line_count: block_lines,
                            max_allowed: self.max_struct_lines,
                            suggestion: "Consider splitting into smaller, focused impl blocks"
                                .to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            if structs_in_file.len() > 5 {
                let struct_names: Vec<String> =
                    structs_in_file.iter().map(|(n, _)| n.clone()).collect();

                if !Self::structs_seem_related(&struct_names) {
                    violations.push(SolidViolation::MultipleUnrelatedStructs {
                        file: path.clone(),
                        struct_names,
                        suggestion: "Consider splitting into separate modules".to_string(),
                        severity: Severity::Info,
                    });
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// OCP: Check for excessive match statements
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails.
    pub fn validate_ocp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let match_pattern = required_pattern("SOLID003.match_keyword")?;

        self.for_each_rust_file(|path, lines| {
            for (line_num, line) in lines.iter().enumerate() {
                if match_pattern.is_match(line) {
                    let arm_count = Self::count_match_arms(&lines, line_num)?;

                    if arm_count > self.max_match_arms {
                        violations.push(SolidViolation::ExcessiveMatchArms {
                            file: path.clone(),
                            line: line_num + 1,
                            arm_count,
                            max_recommended: self.max_match_arms,
                            suggestion:
                                "Consider using visitor pattern, enum dispatch, or trait objects"
                                    .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// ISP: Check for traits with too many methods
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails.
    pub fn validate_isp(&self) -> Result<Vec<SolidViolation>> {
        let trait_pattern = required_pattern("SOLID001.trait_decl")?;
        let fn_pattern = required_pattern("SOLID001.fn_decl")?;

        self.scan_decl_blocks(
            trait_pattern,
            fn_pattern,
            Self::count_trait_methods,
            self.max_trait_methods,
            |file, line, trait_name, method_count, max_allowed| SolidViolation::TraitTooLarge {
                file,
                line,
                trait_name: trait_name.to_string(),
                method_count,
                max_allowed,
                suggestion: "Consider splitting into smaller, focused traits".to_string(),
                severity: Severity::Warning,
            },
        )
    }

    /// LSP: Check for partial trait implementations (panic!/todo! in trait methods).
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails.
    pub fn validate_lsp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_for_pattern = required_pattern("SOLID002.impl_for_decl")?;
        let fn_pattern = required_pattern("SOLID002.fn_decl")?;
        let panic_todo_pattern = required_pattern("SOLID003.panic_macros")?;

        self.for_each_rust_file(|path, lines| {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(cap) = impl_for_pattern.captures(line) {
                    let trait_name = cap.get(1).map_or("", |m| m.as_str());
                    let impl_name = cap.get(2).map_or("", |m| m.as_str());

                    if let Some((block_lines, _)) =
                        crate::scan::extract_balanced_block(&lines, line_num)
                    {
                        let mut current_method: Option<(String, usize)> = None;

                        for (idx, impl_line) in block_lines.iter().enumerate() {
                            if let Some(fn_cap) = fn_pattern.captures(impl_line) {
                                let method_name = fn_cap.get(1).map_or("", |m| m.as_str());
                                current_method =
                                    Some((method_name.to_string(), line_num + idx + 1));
                            }

                            if let Some((ref method_name, method_line)) = current_method
                                && panic_todo_pattern.is_match(impl_line)
                            {
                                violations.push(SolidViolation::PartialTraitImplementation {
                                    file: path.clone(),
                                    line: method_line,
                                    impl_name: format!("{impl_name}::{trait_name}"),
                                    method_name: method_name.clone(),
                                    severity: Severity::Warning,
                                });
                                current_method = None;
                            }
                        }
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// SRP: Check for impl blocks with too many methods
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails.
    pub fn validate_impl_method_count(&self) -> Result<Vec<SolidViolation>> {
        let impl_pattern = required_pattern("SOLID003.impl_only_decl")?;
        let fn_pattern = required_pattern("SOLID002.fn_decl")?;

        self.scan_decl_blocks(
            impl_pattern,
            fn_pattern,
            Self::count_impl_methods,
            self.max_impl_methods,
            |file, line, type_name, method_count, max_allowed| SolidViolation::ImplTooManyMethods {
                file,
                line,
                type_name: type_name.to_string(),
                method_count,
                max_allowed,
                suggestion:
                    "Consider splitting into smaller, focused impl blocks or extracting to traits"
                        .to_string(),
                severity: Severity::Warning,
            },
        )
    }

    /// OCP: Check for string-based type dispatch
    ///
    /// # Errors
    /// Returns an error if pattern compilation fails.
    pub fn validate_string_dispatch(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let string_match_pattern = required_pattern("SOLID003.string_match")?;
        let string_arm_pattern = required_pattern("SOLID003.string_arm")?;

        self.for_each_rust_file(|path, lines| {
            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if string_match_pattern.is_match(line) {
                    let string_arm_count =
                        Self::count_string_match_arms(&lines, line_num, string_arm_pattern);

                    if string_arm_count >= 3 {
                        violations.push(SolidViolation::StringBasedDispatch {
                            file: path.clone(),
                            line: line_num + 1,
                            match_expression: trimmed.chars().take(60).collect(),
                            suggestion:
                                "Consider using enum types with FromStr or a registry pattern"
                                    .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Generic helper: iterate over lines within a brace-delimited block
    fn within_block<F>(lines: &[&str], start_line: usize, mut visitor: F)
    where
        F: FnMut(&str, usize) -> bool,
    {
        let mut brace_depth = 0;
        let mut in_block = false;

        for (idx, line) in lines[start_line..].iter().enumerate() {
            if line.contains('{') {
                in_block = true;
            }
            if in_block {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if !visitor(line, idx) {
                    break;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }
    }

    /// Count methods in an impl block
    fn count_impl_methods(lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut count = 0;
        Self::within_block(lines, start_line, |line, _| {
            if fn_pattern.is_match(line) {
                count += 1;
            }
            true
        });
        count
    }

    /// Count string match arms
    fn count_string_match_arms(
        lines: &[&str],
        start_line: usize,
        string_arm_pattern: &Regex,
    ) -> usize {
        let mut count = 0;
        Self::within_block(lines, start_line, |line, _| {
            if string_arm_pattern.is_match(line) {
                count += 1;
            }
            true
        });
        count
    }

    /// Count lines in a code block (impl, struct, etc.)
    fn count_block_lines(lines: &[&str], start_line: usize) -> usize {
        let mut count = 0;
        Self::within_block(lines, start_line, |_, _| {
            count += 1;
            true
        });
        count
    }

    /// Count match arms in a match statement
    fn count_match_arms(lines: &[&str], start_line: usize) -> Result<usize> {
        let arrow_pattern = required_pattern("SOLID003.match_arrow")?;

        let mut count = 0;
        let mut brace_depth = 0;

        Self::within_block(lines, start_line, |line, _| {
            brace_depth += line.chars().filter(|c| *c == '{').count();
            brace_depth -= line.chars().filter(|c| *c == '}').count();

            if arrow_pattern.is_match(line) && brace_depth >= 1 {
                count += 1;
            }
            true
        });
        Ok(count)
    }

    /// Count methods in a trait definition
    fn count_trait_methods(lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut count = 0;
        Self::within_block(lines, start_line, |line, _| {
            if fn_pattern.is_match(line) {
                count += 1;
            }
            true
        });
        count
    }

    /// Check if structs seem related (share common prefix/suffix).
    fn structs_seem_related(names: &[String]) -> bool {
        if names.len() < 2 {
            return true;
        }

        let checks = [
            Self::has_common_prefix,
            Self::has_common_suffix,
            Self::has_purpose_suffix,
            Self::has_shared_keyword,
            Self::has_common_words,
        ];

        checks.iter().any(|check| check(names))
    }

    /// Check for common prefix (at least 3 chars)
    fn has_common_prefix(names: &[String]) -> bool {
        let first = &names[0];
        for len in (3..=first.len().min(10)).rev() {
            let prefix = &first[..len];
            if names.iter().all(|n| n.starts_with(prefix)) {
                return true;
            }
        }
        false
    }

    /// Check for common suffix (at least 3 chars)
    fn has_common_suffix(names: &[String]) -> bool {
        let first = &names[0];
        for len in (3..=first.len().min(10)).rev() {
            let suffix = &first[first.len().saturating_sub(len)..];
            if names.iter().all(|n| n.ends_with(suffix)) {
                return true;
            }
        }
        false
    }

    /// Check if structs share related purpose suffixes
    fn has_purpose_suffix(names: &[String]) -> bool {
        let purpose_suffixes = [
            "Config",
            "State",
            "Error",
            "Request",
            "Response",
            "Options",
            "Args",
            "Report",
            "Entry",
            "Info",
            "Data",
            "Metrics",
            "Operation",
            "Status",
            "Result",
            "Summary",
            "File",
            "Match",
            "Check",
            "Health",
            "Complexity",
        ];
        names
            .iter()
            .any(|n| purpose_suffixes.iter().any(|suffix| n.ends_with(suffix)))
    }

    /// Check if structs share domain keywords
    fn has_shared_keyword(names: &[String]) -> bool {
        let domain_keywords = [
            "Config",
            "Options",
            "Settings",
            "Error",
            "Result",
            "Builder",
            "Handler",
            "Provider",
            "Service",
            "Health",
            "Crypto",
            "Admin",
            "Http",
            "Args",
            "Request",
            "Response",
            "State",
            "Status",
            "Info",
            "Data",
            "Message",
            "Event",
            "Token",
            "Auth",
            "Cache",
            "Index",
            "Search",
            "Chunk",
            "Embed",
            "Vector",
            "Transport",
            "Operation",
            "Mcp",
            "Protocol",
            "Server",
            "Client",
            "Connection",
            "Session",
            "Route",
            "Endpoint",
            "Memory",
            "Observation",
            "Filter",
            "Pattern",
        ];

        domain_keywords.iter().any(|keyword| {
            let has_keyword: Vec<_> = names.iter().filter(|n| n.contains(keyword)).collect();
            has_keyword.len() > names.len() / 2
        })
    }

    /// Check for partial word overlaps in CamelCase names.
    fn has_common_words(names: &[String]) -> bool {
        let words: Vec<Vec<&str>> = names.iter().map(|n| Self::split_camel_case(n)).collect();

        if let Some(first_words) = words.first() {
            for word in first_words {
                if word.len() >= 4 {
                    let count = words.iter().filter(|w| w.contains(word)).count();
                    if count > names.len() / 2 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Split a CamelCase string into words.
    fn split_camel_case(s: &str) -> Vec<&str> {
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

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }
}

impl_validator!(SolidValidator, "solid", "Validates SOLID principles");
