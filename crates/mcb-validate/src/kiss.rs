//! KISS Principle Validation
//!
//! Validates code against the KISS principle (Keep It Simple, Stupid):
//! - Struct field count (max 12)
//! - Function parameter count (max 5)
//! - Builder complexity (max 7 optional fields)
//! - Nesting depth (max 3 levels)
//! - Function length (max 50 lines)

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::KISSRulesConfig;
use crate::thresholds::thresholds;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// KISS violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KissViolation {
    /// Struct with too many fields (>12)
    StructTooManyFields {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the struct.
        struct_name: String,
        /// Number of fields in the struct.
        field_count: usize,
        /// Maximum allowed fields.
        max_allowed: usize,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Function with too many parameters (>5)
    FunctionTooManyParams {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the function.
        function_name: String,
        /// Number of parameters in the function.
        param_count: usize,
        /// Maximum allowed parameters.
        max_allowed: usize,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Builder with too many optional fields (>7)
    BuilderTooComplex {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the builder struct.
        builder_name: String,
        /// Number of optional fields in the builder.
        optional_field_count: usize,
        /// Maximum allowed optional fields.
        max_allowed: usize,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Nested conditionals too deep (>3 levels)
    DeepNesting {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Current nesting level.
        nesting_level: usize,
        /// Maximum allowed nesting level.
        max_allowed: usize,
        /// Contextual code snippet.
        context: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Function too long (>50 lines)
    FunctionTooLong {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the function.
        function_name: String,
        /// Number of lines in the function.
        line_count: usize,
        /// Maximum allowed lines.
        max_allowed: usize,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl KissViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for KissViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructTooManyFields {
                file,
                line,
                struct_name,
                field_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Struct {} has too many fields: {}:{} ({} fields, max: {})",
                    struct_name,
                    file.display(),
                    line,
                    field_count,
                    max_allowed
                )
            }
            Self::FunctionTooManyParams {
                file,
                line,
                function_name,
                param_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Function {} has too many parameters: {}:{} ({} params, max: {})",
                    function_name,
                    file.display(),
                    line,
                    param_count,
                    max_allowed
                )
            }
            Self::BuilderTooComplex {
                file,
                line,
                builder_name,
                optional_field_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Builder {} is too complex: {}:{} ({} optional fields, max: {})",
                    builder_name,
                    file.display(),
                    line,
                    optional_field_count,
                    max_allowed
                )
            }
            Self::DeepNesting {
                file,
                line,
                nesting_level,
                max_allowed,
                context,
                ..
            } => {
                write!(
                    f,
                    "KISS: Deep nesting at {}:{} ({} levels, max: {}) - {}",
                    file.display(),
                    line,
                    nesting_level,
                    max_allowed,
                    context
                )
            }
            Self::FunctionTooLong {
                file,
                line,
                function_name,
                line_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Function {} is too long: {}:{} ({} lines, max: {})",
                    function_name,
                    file.display(),
                    line,
                    line_count,
                    max_allowed
                )
            }
        }
    }
}

impl Violation for KissViolation {
    fn id(&self) -> &str {
        match self {
            Self::StructTooManyFields { .. } => "KISS001",
            Self::FunctionTooManyParams { .. } => "KISS002",
            Self::BuilderTooComplex { .. } => "KISS003",
            Self::DeepNesting { .. } => "KISS004",
            Self::FunctionTooLong { .. } => "KISS005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Kiss
    }

    fn severity(&self) -> Severity {
        match self {
            Self::StructTooManyFields { severity, .. }
            | Self::FunctionTooManyParams { severity, .. }
            | Self::BuilderTooComplex { severity, .. }
            | Self::DeepNesting { severity, .. }
            | Self::FunctionTooLong { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::StructTooManyFields { file, .. }
            | Self::FunctionTooManyParams { file, .. }
            | Self::BuilderTooComplex { file, .. }
            | Self::DeepNesting { file, .. }
            | Self::FunctionTooLong { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::StructTooManyFields { line, .. }
            | Self::FunctionTooManyParams { line, .. }
            | Self::BuilderTooComplex { line, .. }
            | Self::DeepNesting { line, .. }
            | Self::FunctionTooLong { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::StructTooManyFields {
                struct_name,
                field_count,
                max_allowed,
                ..
            } => Some(format!(
                "Split '{struct_name}' into smaller structs or use composition. \
                 {field_count} fields exceeds the maximum of {max_allowed}."
            )),
            Self::FunctionTooManyParams {
                function_name,
                param_count,
                max_allowed,
                ..
            } => Some(format!(
                "Refactor '{function_name}' to use a config/options struct instead of {param_count} parameters. \
                 Maximum allowed is {max_allowed}."
            )),
            Self::BuilderTooComplex {
                builder_name,
                optional_field_count,
                max_allowed,
                ..
            } => Some(format!(
                "Split '{builder_name}' into smaller builders or use builder composition. \
                 {optional_field_count} optional fields exceeds the maximum of {max_allowed}."
            )),
            Self::DeepNesting {
                nesting_level,
                max_allowed,
                ..
            } => Some(format!(
                "Extract nested logic into separate functions using early returns or guard clauses. \
                 Nesting depth {nesting_level} exceeds the maximum of {max_allowed}."
            )),
            Self::FunctionTooLong {
                function_name,
                line_count,
                max_allowed,
                ..
            } => Some(format!(
                "Break '{function_name}' into smaller, focused functions. \
                 {line_count} lines exceeds the maximum of {max_allowed}."
            )),
        }
    }
}

/// KISS principle validator
pub struct KissValidator {
    config: ValidationConfig,
    rules: KISSRulesConfig,
    /// Maximum allowed fields in a struct.
    max_struct_fields: usize,
    /// Maximum allowed parameters in a function.
    max_function_params: usize,
    /// Maximum allowed optional fields in a builder.
    max_builder_fields: usize,
    /// Maximum allowed nesting depth.
    max_nesting_depth: usize,
    /// Maximum allowed lines in a function.
    max_function_lines: usize,
}

impl KissValidator {
    /// Create a new KISS validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.kiss)
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig, rules: &KISSRulesConfig) -> Self {
        let t = thresholds();
        Self {
            config,
            rules: rules.clone(),
            max_struct_fields: t.max_struct_fields,
            max_function_params: t.max_function_params,
            max_builder_fields: t.max_builder_fields,
            max_nesting_depth: t.max_nesting_depth,
            max_function_lines: t.max_function_lines,
        }
    }

    /// Set custom max struct fields
    #[must_use]
    pub fn with_max_struct_fields(mut self, max: usize) -> Self {
        self.max_struct_fields = max;
        self
    }

    /// Set custom max function parameters
    #[must_use]
    pub fn with_max_function_params(mut self, max: usize) -> Self {
        self.max_function_params = max;
        self
    }

    /// Run all KISS validations
    pub fn validate_all(&self) -> Result<Vec<KissViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_struct_fields()?);
        violations.extend(self.validate_function_params()?);
        violations.extend(self.validate_builder_complexity()?);
        violations.extend(self.validate_nesting_depth()?);
        violations.extend(self.validate_function_length()?);
        Ok(violations)
    }

    /// Check for structs with too many fields
    pub fn validate_struct_fields(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)\s*\{").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
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

                        // DI containers and config structs (ADR-029) have a higher limit
                        // They legitimately aggregate many dependencies, but should still have limits
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

                        // Count fields in struct
                        let field_count = self.count_struct_fields(&lines, line_num);

                        if field_count > max_fields {
                            violations.push(KissViolation::StructTooManyFields {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                struct_name: struct_name.to_string(),
                                field_count,
                                max_allowed: max_fields,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for functions with too many parameters
    pub fn validate_function_params(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = Regex::new(
            r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)",
        )
        .expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip admin API files - builder-like constructors aggregate dependencies
                if path_str.ends_with("/admin/api.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
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

                    // Only check lines that contain function definitions
                    if !line.contains("fn ") {
                        continue;
                    }

                    // Build full function signature (may span multiple lines)
                    let mut full_line = line.to_string();
                    let mut idx = line_num + 1;
                    while !full_line.contains(')') && idx < lines.len() {
                        full_line.push_str(lines[idx]);
                        idx += 1;
                    }

                    if let Some(cap) = fn_pattern.captures(&full_line) {
                        let fn_name = cap.get(1).map_or("", |m| m.as_str());
                        let params = cap.get(2).map_or("", |m| m.as_str());

                        // Count parameters (comma-separated, excluding &self/self)
                        let param_count = self.count_function_params(params);

                        if param_count > self.max_function_params {
                            violations.push(KissViolation::FunctionTooManyParams {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                function_name: fn_name.to_string(),
                                param_count,
                                max_allowed: self.max_function_params,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for builders with too many optional fields
    pub fn validate_builder_complexity(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let builder_pattern = Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Builder)\s*")
            .expect("Invalid regex");
        let option_pattern = Regex::new(r"Option<").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = builder_pattern.captures(line) {
                        let builder_name = cap.get(1).map_or("", |m| m.as_str());

                        // Count Option<> fields in builder struct
                        let optional_count =
                            self.count_optional_fields(&lines, line_num, &option_pattern);

                        if optional_count > self.max_builder_fields {
                            violations.push(KissViolation::BuilderTooComplex {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                builder_name: builder_name.to_string(),
                                optional_field_count: optional_count,
                                max_allowed: self.max_builder_fields,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for deeply nested code
    pub fn validate_nesting_depth(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let control_flow_pattern =
            Regex::new(r"\b(if|match|for|while|loop)\b").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;

                // Track nesting depth
                let mut nesting_depth: usize = 0;
                let mut brace_depth: i32 = 0;
                let mut reported_lines: std::collections::HashSet<usize> =
                    std::collections::HashSet::new();

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        test_brace_depth = brace_depth;
                    }

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track control flow nesting
                    if control_flow_pattern.is_match(line) && line.contains('{') {
                        nesting_depth += 1;

                        // Check if too deep and not already reported nearby
                        if nesting_depth > self.max_nesting_depth {
                            let nearby_reported =
                                reported_lines.iter().any(|&l| l.abs_diff(line_num) < 5);

                            if !nearby_reported {
                                violations.push(KissViolation::DeepNesting {
                                    file: entry.path().to_path_buf(),
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

                    // Decrease nesting on closing braces
                    if close_braces > 0 && nesting_depth > 0 {
                        nesting_depth = nesting_depth.saturating_sub(close_braces);
                    }

                    // Exit test module when braces close
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for functions that are too long
    pub fn validate_function_length(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip DI composition root files (ADR-029)
                // These are legitimately long as they wire up all dependencies
                if path_str.ends_with("/di/bootstrap.rs")
                    || path_str.ends_with("/di/catalog.rs")
                    || path_str.ends_with("/di/resolver.rs")
                {
                    continue;
                }

                // Skip health.rs - system health checks need to collect multiple metrics
                if path_str.ends_with("/health.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
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

                        // Skip test functions
                        if fn_name.starts_with("test_") {
                            continue;
                        }

                        // Skip trait function declarations (no body, ends with ;)
                        if self.is_trait_fn_declaration(&lines, line_num) {
                            continue;
                        }

                        // Count lines in function
                        let line_count = self.count_function_lines(&lines, line_num);

                        if line_count > self.max_function_lines {
                            violations.push(KissViolation::FunctionTooLong {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                function_name: fn_name.to_string(),
                                line_count,
                                max_allowed: self.max_function_lines,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Count fields in a struct
    fn count_struct_fields(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_struct = false;
        let mut field_count = 0;
        let field_pattern =
            Regex::new(r"^\s*(?:pub\s+)?[a-z_][a-z0-9_]*\s*:").expect("Invalid regex");

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_struct = true;
            }
            if in_struct {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count field declarations (lines with `:` that look like fields)
                if brace_depth >= 1 && field_pattern.is_match(line) {
                    field_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        field_count
    }

    /// Count function parameters (excluding self)
    fn count_function_params(&self, params: &str) -> usize {
        if params.trim().is_empty() {
            return 0;
        }

        // Split by comma and count, excluding &self, self, &mut self
        let parts: Vec<&str> = params.split(',').collect();
        let mut count = 0;

        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("&self")
                && !trimmed.starts_with("self")
                && !trimmed.starts_with("&mut self")
            {
                count += 1;
            }
        }

        count
    }

    /// Count Option<> fields in a struct
    fn count_optional_fields(
        &self,
        lines: &[&str],
        start_line: usize,
        option_pattern: &Regex,
    ) -> usize {
        let mut brace_depth = 0;
        let mut in_struct = false;
        let mut optional_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_struct = true;
            }
            if in_struct {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count Option<> types
                if brace_depth >= 1 && option_pattern.is_match(line) {
                    optional_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        optional_count
    }

    /// Check if a function declaration is a trait method without a body
    /// (ends with `;` before any `{`)
    fn is_trait_fn_declaration(&self, lines: &[&str], start_line: usize) -> bool {
        // Look at the function signature lines until we find either { or ;
        // If we find ; first, it's a trait function declaration without a body
        for line in &lines[start_line..] {
            // Check for opening brace (function body starts)
            if line.contains('{') {
                return false;
            }
            // Check for semicolon (trait function declaration ends)
            if line.trim().ends_with(';') {
                return true;
            }
            // Check for semicolon after return type annotation
            // e.g., "fn foo(&self) -> Result<T>;"
            if line.contains(';') && !line.contains('{') {
                return true;
            }
        }
        false
    }

    /// Count lines in a function
    fn count_function_lines(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_fn = false;
        let mut line_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_fn = true;
            }
            if in_fn {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();
                line_count += 1;

                if brace_depth == 0 {
                    break;
                }
            }
        }

        line_count
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
    KissValidator,
    "kiss",
    "Validates KISS principle (Keep It Simple, Stupid)"
);
