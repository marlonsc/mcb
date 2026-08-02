use crate::filters::LanguageId;
use regex::Regex;
use std::path::{Path, PathBuf};

use super::helpers::{
    DocItemContext, DocRegexContext, MissingDocSpec, ScanLineContext, SimplePubItemSpec,
    get_doc_comment_section, has_doc_comment,
};
use crate::define_violations;
use crate::scan::for_each_crate_file;
use crate::{Result, Severity, ValidationConfig};
use mcb_domain::ports::validation::ViolationCategory;
use mcb_utils::constants::validate::{
    ATTR_REGEX, DI_MODULES_PATH, DOC_COMMENT_CAPTURE_REGEX, DOC_COMMENT_REGEX,
    EXAMPLE_SECTION_REGEX, ITEM_KIND_ENUM, ITEM_KIND_FUNCTION, ITEM_KIND_STRUCT, ITEM_KIND_TRAIT,
    MODULE_DOC_REGEX, MODULE_FILE_NAMES, PORTS_DIR, PUB_ENUM_REGEX, PUB_FN_REGEX, PUB_STRUCT_REGEX,
    PUB_TRAIT_REGEX,
};
use mcb_utils::utils::regex::compile_regex;

/// Compiled regexes for scanning public items that require documentation.
struct PubDocPatterns {
    pub_struct: Regex,
    pub_enum: Regex,
    pub_trait: Regex,
    pub_fn: Regex,
    example: Regex,
    doc_comment: Regex,
    doc_comment_capture: Regex,
    attr: Regex,
    /// Items declared inside a macro invocation (e.g. `define_violations! { pub enum ... }`)
    /// are documented by the macro expansion, not the source text, so the text
    /// scanner must skip them.
    macro_invocation: Regex,
}

impl PubDocPatterns {
    /// Compile every pattern used by the public-item documentation scan.
    fn compile() -> Result<Self> {
        Ok(Self {
            pub_struct: compile_regex(PUB_STRUCT_REGEX)?,
            pub_enum: compile_regex(PUB_ENUM_REGEX)?,
            pub_trait: compile_regex(PUB_TRAIT_REGEX)?,
            pub_fn: compile_regex(PUB_FN_REGEX)?,
            example: compile_regex(EXAMPLE_SECTION_REGEX)?,
            doc_comment: compile_regex(DOC_COMMENT_REGEX)?,
            doc_comment_capture: compile_regex(DOC_COMMENT_CAPTURE_REGEX)?,
            attr: compile_regex(ATTR_REGEX)?,
            macro_invocation: compile_regex(r"^\s*[a-z_][a-z0-9_]*!\s*\{")?,
        })
    }

    /// Borrowed regex bundle for doc-comment detection helpers.
    fn regex_context(&self) -> DocRegexContext<'_> {
        DocRegexContext {
            doc_comment_re: &self.doc_comment,
            doc_comment_capture_re: &self.doc_comment_capture,
            attr_re: &self.attr,
            example_pattern: &self.example,
        }
    }

    /// Specs for simple public items (struct, enum) checked uniformly.
    fn simple_item_specs(&self) -> [SimplePubItemSpec<'_>; 2] {
        [
            SimplePubItemSpec {
                pattern: &self.pub_struct,
                item_kind: ITEM_KIND_STRUCT,
            },
            SimplePubItemSpec {
                pattern: &self.pub_enum,
                item_kind: ITEM_KIND_ENUM,
            },
        ]
    }
}

define_violations! {
    dynamic_severity,
    ViolationCategory::Documentation,
    pub enum DocumentationViolation {
        /// Missing module-level documentation
        #[violation(
            id = "DOC001",
            severity = Warning,
            message = "Missing module doc: {file}",
            suggestion = "Add //! module-level documentation at the top of the file"
        )]
        MissingModuleDoc {
            file: PathBuf,
            severity: Severity,
        },
        /// Missing documentation on public item
        #[violation(
            id = "DOC002",
            severity = Warning,
            message = "Missing {item_kind} doc: {file}:{line} - {item_name}",
            suggestion = "Add /// documentation for {item_kind} {item_name}"
        )]
        MissingPubItemDoc {
            file: PathBuf,
            line: usize,
            item_name: String,
            item_kind: String,
            severity: Severity,
        },
        /// Missing example code in documentation
        #[violation(
            id = "DOC003",
            severity = Info,
            message = "Missing example: {file}:{line} - {item_name}",
            suggestion = "Add # Example section to {item_name} documentation"
        )]
        MissingExampleCode {
            file: PathBuf,
            line: usize,
            item_name: String,
            item_kind: String,
            severity: Severity,
        },
    }
}

crate::create_validator!(
    DocumentationValidator,
    mcb_utils::constants::validate::VALIDATOR_DOCUMENTATION,
    "Validates documentation standards",
    DocumentationViolation,
    [Self::validate_module_docs, Self::validate_pub_item_docs,]
);

impl DocumentationValidator {
    /// Verify module-level documentation exists
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation or file scanning fails.
    pub fn validate_module_docs(config: &ValidationConfig) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        let module_doc_pattern = compile_regex(MODULE_DOC_REGEX)?;

        for_each_crate_file(
            config,
            Some(LanguageId::Rust),
            |entry, _src_dir, _crate_name| {
                let path = &entry.absolute_path;
                let content = std::fs::read_to_string(path)?;
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                // Only check module files that require documentation
                if !MODULE_FILE_NAMES.contains(&file_name) {
                    return Ok(());
                }

                // Check if first non-empty line is module doc
                let has_module_doc = content
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .take(1)
                    .any(|line| module_doc_pattern.is_match(line));

                if !has_module_doc {
                    violations.push(DocumentationViolation::MissingModuleDoc {
                        file: path.clone(),
                        severity: Severity::Warning,
                    });
                }

                Ok(())
            },
        )?;

        Ok(violations)
    }

    /// Verify all pub items have rustdoc
    ///
    /// # Errors
    ///
    /// Returns an error if regex compilation or file scanning fails.
    pub fn validate_pub_item_docs(
        config: &ValidationConfig,
    ) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        let patterns = PubDocPatterns::compile()?;

        for_each_crate_file(
            config,
            Some(LanguageId::Rust),
            |entry, _src_dir, _crate_name| {
                let content = std::fs::read_to_string(&entry.absolute_path)?;
                let lines: Vec<&str> = content.lines().collect();
                Self::scan_file_for_pub_docs(
                    &entry.absolute_path,
                    &lines,
                    &patterns,
                    &mut violations,
                );
                Ok(())
            },
        )?;

        Ok(violations)
    }

    /// Scan a single file's lines for undocumented public items.
    fn scan_file_for_pub_docs(
        path: &Path,
        lines: &[&str],
        patterns: &PubDocPatterns,
        violations: &mut Vec<DocumentationViolation>,
    ) {
        // INTENTIONAL: Path to string; non-UTF8 paths yield empty string (best-effort)
        let path_str = path.to_str().unwrap_or_default();
        let regex_ctx = patterns.regex_context();
        let simple_pub_item_specs = patterns.simple_item_specs();

        let mut macro_depth: i32 = 0;
        for (line_num, line) in lines.iter().enumerate() {
            if Self::advance_macro_depth(&mut macro_depth, line, &patterns.macro_invocation) {
                continue;
            }

            let scan_ctx = ScanLineContext {
                path,
                lines,
                line_num,
                line,
            };
            Self::check_simple_public_item_docs(
                violations,
                &scan_ctx,
                &simple_pub_item_specs,
                &regex_ctx,
            );
            Self::check_trait_and_fn_docs(
                violations,
                &scan_ctx,
                path_str,
                &regex_ctx,
                (&patterns.pub_trait, &patterns.pub_fn),
            );
        }
    }

    /// Update `macro_depth` for `line` and return `true` when the line is inside
    /// (or opening) a macro-invocation block whose pub items must be skipped.
    fn advance_macro_depth(
        macro_depth: &mut i32,
        line: &str,
        macro_invocation_re: &regex::Regex,
    ) -> bool {
        let entering_macro = *macro_depth == 0
            && macro_invocation_re.is_match(line)
            && !line.contains("macro_rules!");
        if *macro_depth > 0 || entering_macro {
            *macro_depth += i32::try_from(line.matches('{').count()).unwrap_or(0);
            *macro_depth -= i32::try_from(line.matches('}').count()).unwrap_or(0);
            *macro_depth = (*macro_depth).max(0);
            return true;
        }
        false
    }

    /// Check the current line for an undocumented public trait and/or function,
    /// appending any violations found.
    fn check_trait_and_fn_docs(
        violations: &mut Vec<DocumentationViolation>,
        scan_ctx: &ScanLineContext<'_>,
        path_str: &str,
        regex_ctx: &DocRegexContext<'_>,
        patterns: (&regex::Regex, &regex::Regex),
    ) {
        let (pub_trait_pattern, pub_fn_pattern) = patterns;
        if let Some(cap) = pub_trait_pattern.captures(scan_ctx.line) {
            let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
            let item_ctx =
                Self::build_item_ctx(scan_ctx.path, scan_ctx.lines, scan_ctx.line_num, name);
            Self::check_public_trait_docs(violations, &item_ctx, path_str, regex_ctx);
        }

        if let Some(cap) = pub_fn_pattern.captures(scan_ctx.line) {
            let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
            let item_ctx =
                Self::build_item_ctx(scan_ctx.path, scan_ctx.lines, scan_ctx.line_num, name);
            Self::check_public_function_docs(violations, &item_ctx, scan_ctx.line, regex_ctx);
        }
    }

    fn build_item_ctx<'a>(
        path: &'a std::path::Path,
        lines: &'a [&'a str],
        line_num: usize,
        item_name: &'a str,
    ) -> DocItemContext<'a> {
        DocItemContext {
            path,
            lines,
            line_num,
            item_name,
        }
    }

    fn check_simple_public_item_docs(
        violations: &mut Vec<DocumentationViolation>,
        scan_ctx: &ScanLineContext<'_>,
        specs: &[SimplePubItemSpec<'_>],
        regex_ctx: &DocRegexContext<'_>,
    ) {
        for spec in specs {
            let Some(cap) = spec.pattern.captures(scan_ctx.line) else {
                continue;
            };

            let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
            let item_ctx =
                Self::build_item_ctx(scan_ctx.path, scan_ctx.lines, scan_ctx.line_num, name);
            Self::push_missing_pub_item_doc_if_needed(
                violations,
                &item_ctx,
                &MissingDocSpec {
                    item_kind: spec.item_kind,
                    severity: Severity::Warning,
                },
                regex_ctx,
            );
        }
    }

    fn push_missing_pub_item_doc_if_needed(
        violations: &mut Vec<DocumentationViolation>,
        item_ctx: &DocItemContext<'_>,
        spec: &MissingDocSpec<'_>,
        regex_ctx: &DocRegexContext<'_>,
    ) {
        if has_doc_comment(
            item_ctx.lines,
            item_ctx.line_num,
            regex_ctx.doc_comment_re,
            regex_ctx.attr_re,
        ) {
            return;
        }

        violations.push(DocumentationViolation::MissingPubItemDoc {
            file: item_ctx.path.to_path_buf(),
            line: item_ctx.line_num + 1,
            item_name: item_ctx.item_name.to_owned(),
            item_kind: spec.item_kind.to_owned(),
            severity: spec.severity,
        });
    }

    fn check_public_trait_docs(
        violations: &mut Vec<DocumentationViolation>,
        item_ctx: &DocItemContext<'_>,
        path_str: &str,
        regex_ctx: &DocRegexContext<'_>,
    ) {
        if !has_doc_comment(
            item_ctx.lines,
            item_ctx.line_num,
            regex_ctx.doc_comment_re,
            regex_ctx.attr_re,
        ) {
            Self::push_missing_pub_item_doc_if_needed(
                violations,
                item_ctx,
                &MissingDocSpec {
                    item_kind: ITEM_KIND_TRAIT,
                    severity: Severity::Warning,
                },
                regex_ctx,
            );
            return;
        }

        let is_di_or_port_trait =
            path_str.contains(DI_MODULES_PATH) || path_str.contains(PORTS_DIR);
        if is_di_or_port_trait {
            return;
        }

        let doc_section = get_doc_comment_section(
            item_ctx.lines,
            item_ctx.line_num,
            regex_ctx.doc_comment_capture_re,
            regex_ctx.attr_re,
        );
        if regex_ctx.example_pattern.is_match(&doc_section) {
            return;
        }

        violations.push(DocumentationViolation::MissingExampleCode {
            file: item_ctx.path.to_path_buf(),
            line: item_ctx.line_num + 1,
            item_name: item_ctx.item_name.to_owned(),
            item_kind: ITEM_KIND_TRAIT.to_owned(),
            severity: Severity::Info,
        });
    }

    fn check_public_function_docs(
        violations: &mut Vec<DocumentationViolation>,
        item_ctx: &DocItemContext<'_>,
        line: &str,
        regex_ctx: &DocRegexContext<'_>,
    ) {
        if line.starts_with("    ") || line.starts_with('\t') {
            return;
        }

        Self::push_missing_pub_item_doc_if_needed(
            violations,
            item_ctx,
            &MissingDocSpec {
                item_kind: ITEM_KIND_FUNCTION,
                severity: Severity::Info,
            },
            regex_ctx,
        );
    }
}
