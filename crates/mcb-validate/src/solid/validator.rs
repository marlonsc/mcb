//! Solid validator implementation

use std::path::PathBuf;

use regex::Regex;
use walkdir::WalkDir;

use super::violation::SolidViolation;
use crate::pattern_registry::PATTERNS;
use crate::thresholds::{MAX_IMPL_METHODS, MAX_MATCH_ARMS, MAX_STRUCT_LINES, MAX_TRAIT_METHODS};
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
        Self {
            config,
            max_trait_methods: MAX_TRAIT_METHODS,
            max_struct_lines: MAX_STRUCT_LINES,
            max_match_arms: MAX_MATCH_ARMS,
            max_impl_methods: MAX_IMPL_METHODS,
        }
    }

    /// Run all SOLID validations
    pub fn validate_all(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_srp()?);
        violations.extend(self.validate_ocp()?);
        violations.extend(self.validate_isp()?);
        violations.extend(self.validate_lsp()?);
        violations.extend(self.validate_impl_method_count()?);
        violations.extend(self.validate_string_dispatch()?);
        Ok(violations)
    }

    /// SRP: Check for structs/impls that are too large
    pub fn validate_srp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_pattern = PATTERNS
            .get("SOLID002.impl_decl")
            .ok_or_else(|| crate::ValidationError::PatternNotFound("SOLID002.impl_decl".into()))?;
        let struct_pattern = PATTERNS.get("SOLID002.struct_decl").ok_or_else(|| {
            crate::ValidationError::PatternNotFound("SOLID002.struct_decl".into())
        })?;

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track struct definitions and their sizes
                let mut structs_in_file: Vec<(String, usize)> = Vec::new();

                for (line_num, line) in lines.iter().enumerate() {
                    // Check struct sizes
                    if let Some(cap) = struct_pattern.captures(line) {
                        let name = cap.get(1).map_or("", |m| m.as_str());
                        structs_in_file.push((name.to_string(), line_num + 1));
                    }

                    // Check impl block sizes
                    if let Some(cap) = impl_pattern.captures(line) {
                        let name = cap.get(1).or(cap.get(2)).map_or("", |m| m.as_str());

                        // Count lines in impl block
                        let block_lines = self.count_block_lines(&lines, line_num);

                        if block_lines > self.max_struct_lines {
                            violations.push(SolidViolation::TooManyResponsibilities {
                                file: entry.path().to_path_buf(),
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

                // Check if file has many unrelated structs (potential SRP violation)
                // Skip collection files which intentionally group related types
                if structs_in_file.len() > 5 {
                    let struct_names: Vec<String> =
                        structs_in_file.iter().map(|(n, _)| n.clone()).collect();

                    // Check if structs seem unrelated (different prefixes/suffixes)
                    if !self.structs_seem_related(&struct_names) {
                        violations.push(SolidViolation::MultipleUnrelatedStructs {
                            file: entry.path().to_path_buf(),
                            struct_names,
                            suggestion: "Consider splitting into separate modules".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// OCP: Check for excessive match statements
    pub fn validate_ocp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let match_pattern = PATTERNS
            .get("SOLID003.match_keyword")
            .expect("Pattern SOLID003.match_keyword not found");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if match_pattern.is_match(line) {
                        // Count match arms
                        let arm_count = self.count_match_arms(&lines, line_num);

                        if arm_count > self.max_match_arms {
                            violations.push(SolidViolation::ExcessiveMatchArms {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                arm_count,
                                max_recommended: self.max_match_arms,
                                suggestion: "Consider using visitor pattern, enum dispatch, or trait objects".to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// ISP: Check for traits with too many methods
    pub fn validate_isp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let trait_pattern = PATTERNS
            .get("SOLID001.trait_decl")
            .expect("Pattern SOLID001.trait_decl not found");
        let fn_pattern = PATTERNS
            .get("SOLID001.fn_decl")
            .expect("Pattern SOLID001.fn_decl not found");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map_or("", |m| m.as_str());

                        // Count methods in trait
                        let method_count = self.count_trait_methods(&lines, line_num, fn_pattern);

                        if method_count > self.max_trait_methods {
                            violations.push(SolidViolation::TraitTooLarge {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                method_count,
                                max_allowed: self.max_trait_methods,
                                suggestion: "Consider splitting into smaller, focused traits"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// LSP: Check for partial trait implementations (panic!/todo! in trait methods)
    pub fn validate_lsp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_for_pattern = PATTERNS
            .get("SOLID002.impl_for_decl")
            .expect("Pattern SOLID002.impl_for_decl not found");
        let fn_pattern = PATTERNS
            .get("SOLID002.fn_decl")
            .expect("Pattern SOLID002.fn_decl not found");
        let panic_todo_pattern = PATTERNS
            .get("SOLID003.panic_macros")
            .expect("Pattern SOLID003.panic_macros not found");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = impl_for_pattern.captures(line) {
                        let trait_name = cap.get(1).map_or("", |m| m.as_str());
                        let impl_name = cap.get(2).map_or("", |m| m.as_str());

                        // Check methods in impl block for panic!/unimplemented macros
                        let mut brace_depth = 0;
                        let mut in_impl = false;
                        let mut current_method: Option<(String, usize)> = None;

                        for (idx, impl_line) in lines[line_num..].iter().enumerate() {
                            if impl_line.contains('{') {
                                in_impl = true;
                            }
                            if in_impl {
                                brace_depth += impl_line.chars().filter(|c| *c == '{').count();
                                brace_depth -= impl_line.chars().filter(|c| *c == '}').count();

                                // Track current method
                                if let Some(fn_cap) = fn_pattern.captures(impl_line) {
                                    let method_name = fn_cap.get(1).map_or("", |m| m.as_str());
                                    current_method =
                                        Some((method_name.to_string(), line_num + idx + 1));
                                }

                                // Check for panic!/todo! in method body
                                if let Some((ref method_name, method_line)) = current_method
                                    && panic_todo_pattern.is_match(impl_line)
                                {
                                    violations.push(SolidViolation::PartialTraitImplementation {
                                        file: entry.path().to_path_buf(),
                                        line: method_line,
                                        impl_name: format!("{impl_name}::{trait_name}"),
                                        method_name: method_name.clone(),
                                        severity: Severity::Warning,
                                    });
                                    current_method = None; // Don't report same method twice
                                }

                                if brace_depth == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// SRP: Check for impl blocks with too many methods
    pub fn validate_impl_method_count(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_pattern = PATTERNS
            .get("SOLID003.impl_only_decl")
            .expect("Pattern SOLID003.impl_only_decl not found");
        let fn_pattern = PATTERNS
            .get("SOLID002.fn_decl")
            .expect("Pattern SOLID002.fn_decl not found");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = impl_pattern.captures(line) {
                        let type_name = cap.get(1).map_or("", |m| m.as_str());

                        // Count methods in impl block
                        let method_count = self.count_impl_methods(&lines, line_num, fn_pattern);

                        if method_count > self.max_impl_methods {
                            violations.push(SolidViolation::ImplTooManyMethods {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                type_name: type_name.to_string(),
                                method_count,
                                max_allowed: self.max_impl_methods,
                                suggestion: "Consider splitting into smaller, focused impl blocks or extracting to traits".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// OCP: Check for string-based type dispatch
    pub fn validate_string_dispatch(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        // Pattern: match on .as_str() or match with string literals
        let string_match_pattern = PATTERNS
            .get("SOLID003.string_match")
            .expect("Pattern SOLID003.string_match not found");
        let string_arm_pattern = PATTERNS
            .get("SOLID003.string_arm")
            .expect("Pattern SOLID003.string_arm not found");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Check for string-based match dispatch
                    if string_match_pattern.is_match(line) {
                        // Count string arms in the match
                        let string_arm_count =
                            self.count_string_match_arms(&lines, line_num, string_arm_pattern);

                        if string_arm_count >= 3 {
                            violations.push(SolidViolation::StringBasedDispatch {
                                file: entry.path().to_path_buf(),
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
            }
        }

        Ok(violations)
    }

    /// Count methods in an impl block
    fn count_impl_methods(&self, lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut brace_depth = 0;
        let mut in_impl = false;
        let mut method_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_impl = true;
            }
            if in_impl {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if fn_pattern.is_match(line) {
                    method_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        method_count
    }

    /// Count string match arms
    fn count_string_match_arms(
        &self,
        lines: &[&str],
        start_line: usize,
        string_arm_pattern: &Regex,
    ) -> usize {
        let mut brace_depth = 0;
        let mut in_match = false;
        let mut arm_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_match = true;
            }
            if in_match {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if string_arm_pattern.is_match(line) {
                    arm_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        arm_count
    }

    /// Count lines in a code block (impl, struct, etc.)
    fn count_block_lines(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_block = true;
            }
            if in_block {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();
                count += 1;

                if brace_depth == 0 {
                    break;
                }
            }
        }

        count
    }

    /// Count match arms in a match statement
    fn count_match_arms(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_match = false;
        let mut arm_count = 0;
        let arrow_pattern = PATTERNS
            .get("SOLID003.match_arrow")
            .expect("Pattern SOLID003.match_arrow not found");

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_match = true;
            }
            if in_match {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count arrows (match arms)
                if arrow_pattern.is_match(line) && brace_depth >= 1 {
                    arm_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        arm_count
    }

    /// Count methods in a trait definition
    fn count_trait_methods(&self, lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut brace_depth = 0;
        let mut in_trait = false;
        let mut method_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_trait = true;
            }
            if in_trait {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if fn_pattern.is_match(line) {
                    method_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        method_count
    }

    /// Check if structs seem related (share common prefix/suffix)
    #[allow(clippy::too_many_lines)]
    fn structs_seem_related(&self, names: &[String]) -> bool {
        if names.len() < 2 {
            return true;
        }

        Self::has_common_prefix(names)
            || Self::has_common_suffix(names)
            || Self::has_purpose_suffix(names)
            || Self::has_shared_keyword(names)
            || Self::has_common_words(names)
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

    /// Check for partial word overlaps in CamelCase names
    fn has_common_words(names: &[String]) -> bool {
        let words: Vec<Vec<&str>> = names
            .iter()
            .map(|n| {
                let mut words = Vec::new();
                let mut start = 0;
                for (i, c) in n.char_indices() {
                    if c.is_uppercase() && i > 0 {
                        if start < i {
                            words.push(&n[start..i]);
                        }
                        start = i;
                    }
                }
                if start < n.len() {
                    words.push(&n[start..]);
                }
                words
            })
            .collect();

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

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }
}

impl_validator!(SolidValidator, "solid", "Validates SOLID principles");
