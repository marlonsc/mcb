//! Implementation validator implementation

use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use super::violation::ImplementationViolation;
use crate::config::ImplementationRulesConfig;
use crate::pattern_registry::PATTERNS;
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

    /// Iterate over all non-test `.rs` source files in the configured
    /// scan dirs, yielding `(DirEntry, file-content)` pairs.
    fn for_each_source_file<F>(&self, mut visitor: F) -> Result<Vec<ImplementationViolation>>
    where
        F: FnMut(&DirEntry, &str) -> Vec<ImplementationViolation>,
    {
        let mut violations = Vec::new();
        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in walk_rs_files(&src_dir) {
                if is_test_path(&entry) {
                    continue;
                }
                let content = std::fs::read_to_string(entry.path())?;
                violations.extend(visitor(&entry, &content));
            }
        }
        Ok(violations)
    }

    /// Like [`for_each_source_file`] but also skips null/fake provider files.
    fn for_each_prod_source_file<F>(&self, mut visitor: F) -> Result<Vec<ImplementationViolation>>
    where
        F: FnMut(&DirEntry, &str) -> Vec<ImplementationViolation>,
    {
        self.for_each_source_file(|entry, content| {
            let fname = file_name_str(entry);
            if fname.contains("null") || fname.contains("fake") {
                return Vec::new();
            }
            visitor(entry, content)
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

        let fn_pattern = PATTERNS.get("IMPL001.fn_decl");
        let compiled: Vec<_> = compile_pattern_pairs(&empty_pattern_ids);

        self.for_each_prod_source_file(|entry, content| {
            let mut violations = Vec::new();
            let mut current_fn_name = String::new();
            for (line_num, trimmed) in source_lines(content) {
                track_fn_name(fn_pattern, trimmed, &mut current_fn_name);
                for (pattern, desc) in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::EmptyMethodBody {
                            file: entry.path().to_path_buf(),
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

        let fn_pattern = PATTERNS.get("IMPL001.fn_decl");
        let compiled: Vec<_> = compile_pattern_pairs(&hardcoded_pattern_ids);

        self.for_each_prod_source_file(|entry, content| {
            let fname = file_name_str(entry);
            if fname == "constants.rs" {
                return Vec::new();
            }

            let mut violations = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            let non_test_lines = non_test_lines(&lines);

            for func in extract_functions(fn_pattern, &non_test_lines) {
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
                                file: entry.path().to_path_buf(),
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

        let fn_pattern = PATTERNS.get("IMPL001.fn_decl");
        let compiled: Vec<_> = compile_pattern_pairs(&stub_pattern_ids);

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            let mut current_fn_name = String::new();
            for (line_num, trimmed) in source_lines(content) {
                track_fn_name(fn_pattern, trimmed, &mut current_fn_name);
                for (pattern, macro_type) in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::StubMacro {
                            file: entry.path().to_path_buf(),
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

        let compiled: Vec<_> = catchall_ids
            .iter()
            .filter_map(|id| PATTERNS.get(id))
            .collect();

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            for (line_num, trimmed) in source_lines(content) {
                for pattern in &compiled {
                    if pattern.is_match(trimmed) {
                        violations.push(ImplementationViolation::EmptyCatchAll {
                            file: entry.path().to_path_buf(),
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
        let passthrough_pattern = PATTERNS.get("IMPL001.passthrough");
        let fn_pattern = PATTERNS.get("IMPL001.fn_decl");
        let impl_pattern = PATTERNS.get("IMPL001.impl_decl");

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
                fn_pattern,
                impl_pattern,
                &non_test,
                &mut current_struct_name,
            ) {
                if func.meaningful_body.len() != 1 {
                    continue;
                }
                if let Some(re) = passthrough_pattern
                    && let Some(cap) = re.captures(&func.meaningful_body[0])
                {
                    let field = cap.get(1).map_or("", |m| m.as_str());
                    let method = cap.get(2).map_or("", |m| m.as_str());
                    if method == func.name || method.starts_with(&func.name) {
                        violations.push(ImplementationViolation::PassThroughWrapper {
                            file: entry.path().to_path_buf(),
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

        let fn_pattern = PATTERNS.get("IMPL001.fn_decl");
        let compiled_log: Vec<_> = log_pattern_ids
            .iter()
            .filter_map(|id| PATTERNS.get(id))
            .collect();

        self.for_each_source_file(|entry, content| {
            let mut violations = Vec::new();
            let lines: Vec<&str> = content.lines().collect();
            let non_test = non_test_lines(&lines);
            let mut dummy = String::new();

            for func in extract_functions_with_body(fn_pattern, None, &non_test, &mut dummy) {
                if func.meaningful_body.is_empty() || func.meaningful_body.len() > 3 {
                    continue;
                }
                let all_logging = func
                    .meaningful_body
                    .iter()
                    .all(|line| compiled_log.iter().any(|p| p.is_match(line)));
                if all_logging {
                    violations.push(ImplementationViolation::LogOnlyMethod {
                        file: entry.path().to_path_buf(),
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

/// Walk a directory for `.rs` files, skipping walk errors.
fn walk_rs_files(dir: &Path) -> impl Iterator<Item = DirEntry> + '_ {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
}

/// Check if a `DirEntry` is under a test path.
fn is_test_path(entry: &DirEntry) -> bool {
    let path = entry.path().to_string_lossy();
    path.contains("/tests/")
}

/// Get the file name of a `DirEntry` as `&str`.
fn file_name_str(entry: &DirEntry) -> &str {
    entry
        .path()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
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
fn compile_pattern_pairs<'a>(ids: &[(&str, &'a str)]) -> Vec<(&'a Regex, &'a str)> {
    ids.iter()
        .filter_map(|(id, desc)| PATTERNS.get(id).map(|r| (r, *desc)))
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

            for (j, (_, line_content)) in lines.iter().enumerate().skip(i) {
                let opens = i32::try_from(line_content.chars().filter(|c| *c == '{').count())
                    .unwrap_or(i32::MAX);
                let closes = i32::try_from(line_content.chars().filter(|c| *c == '}').count())
                    .unwrap_or(i32::MAX);

                if opens > 0 {
                    fn_started = true;
                }
                brace_depth += opens - closes;
                if fn_started && brace_depth <= 0 {
                    fn_end_idx = j;
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
        let crate_dir = temp.path().join("crates").join(name).join("src");
        fs::create_dir_all(&crate_dir).unwrap();

        // Create a minimal Cargo.toml
        let cargo_toml = temp.path().join("crates").join(name).join("Cargo.toml");
        fs::write(
            cargo_toml,
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n[lib]\npath = \"src/lib.rs\"\n"
            ),
        )
        .unwrap();
        let lib_rs = crate_dir.join("lib.rs");
        fs::write(lib_rs, content).unwrap();
    }

    #[test]
    fn test_empty_method_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "test-crate",
            r#"
pub struct Foo;
impl Foo {
    fn do_something(&self) -> Option<String> {
        None
    }
}
"#,
        );
        let config = ValidationConfig::new(temp.path());
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        let validator = ImplementationQualityValidator::with_config(config, &rules);
        let violations = validator.validate_empty_methods().unwrap();
        assert!(
            !violations.is_empty(),
            "Should detect empty method returning None"
        );
    }

    #[test]
    fn test_hardcoded_return_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "test-crate",
            r#"
pub fn always_true() -> bool {
    true
}
"#,
        );
        let config = ValidationConfig::new(temp.path());
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        let validator = ImplementationQualityValidator::with_config(config, &rules);
        let violations = validator.validate_hardcoded_returns().unwrap();
        assert!(
            !violations.is_empty(),
            "Should detect hardcoded return true"
        );
    }

    #[test]
    fn test_stub_macro_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "test-crate",
            r#"
pub fn not_ready() {
    todo!()
}
pub fn also_not_ready() {
    unimplemented!()
}
"#,
        );
        let config = ValidationConfig::new(temp.path());
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        let validator = ImplementationQualityValidator::with_config(config, &rules);
        let violations = validator.validate_stub_macros().unwrap();
        assert!(
            violations.len() >= 2,
            "Should detect both todo! and unimplemented!"
        );
    }

    #[test]
    fn test_empty_catchall_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "test-crate",
            r#"
pub fn handle(val: i32) {
    match val {
        1 => println!("one"),
        _ => {}
    }
}
"#,
        );
        let config = ValidationConfig::new(temp.path());
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        let validator = ImplementationQualityValidator::with_config(config, &rules);
        let violations = validator.validate_empty_catch_alls().unwrap();
        assert!(
            !violations.is_empty(),
            "Should detect empty catch-all: _ => {{}}"
        );
    }

    #[test]
    fn test_null_provider_exempt() {
        let temp = TempDir::new().unwrap();
        let crate_dir = temp.path().join("crates").join("test-crate").join("src");
        fs::create_dir_all(&crate_dir).unwrap();

        // Create a minimal Cargo.toml
        let cargo_toml = temp
            .path()
            .join("crates")
            .join("test-crate")
            .join("Cargo.toml");
        fs::write(
            cargo_toml,
            "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .unwrap();

        // Regular lib.rs
        fs::write(crate_dir.join("lib.rs"), "pub mod null_provider;\n").unwrap();

        // Null provider file - should be exempt from empty method detection
        fs::write(
            crate_dir.join("null_provider.rs"),
            r#"
pub struct NullVectorStore;
impl NullVectorStore {
    pub fn search(&self) -> Vec<String> {
        Vec::new()
    }
    pub fn store(&self) -> Result<(), String> {
        Ok(())
    }
}
"#,
        )
        .unwrap();

        let config = ValidationConfig::new(temp.path());
        let rules = ImplementationRulesConfig {
            enabled: true,
            excluded_crates: Vec::new(),
        };
        let validator = ImplementationQualityValidator::with_config(config, &rules);
        let violations = validator.validate_empty_methods().unwrap();
        assert!(
            violations.is_empty(),
            "Null provider files should be exempt from empty method detection, got: {violations:?}"
        );
    }
}
