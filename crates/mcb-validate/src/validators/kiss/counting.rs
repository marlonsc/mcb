//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use regex::Regex;

use super::KissValidator;
use crate::constants::kiss::SELF_PARAM_VARIANTS;
use crate::pattern_registry::compile_regex;

impl KissValidator {
    fn count_block_matches(
        lines: &[&str],
        start_line: usize,
        predicate: impl Fn(&str) -> bool,
    ) -> usize {
        let mut brace_depth = 0usize;
        let mut in_block = false;
        let mut count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_block = true;
            }
            if in_block {
                brace_depth += line.chars().filter(|ch| *ch == '{').count();
                brace_depth -= line.chars().filter(|ch| *ch == '}').count();
                if brace_depth >= 1 && predicate(line) {
                    count += 1;
                }
                if brace_depth == 0 {
                    break;
                }
            }
        }

        count
    }

    pub(super) fn count_struct_fields(lines: &[&str], start_line: usize) -> usize {
        let field_pattern = match compile_regex(r"^\s*(?:pub\s+)?[a-z_][a-z0-9_]*\s*:") {
            Ok(regex) => regex,
            Err(_) => return 0,
        };

        Self::count_block_matches(lines, start_line, |line| field_pattern.is_match(line))
    }

    pub(super) fn count_function_params(params: &str) -> usize {
        if params.trim().is_empty() {
            return 0;
        }

        let parts: Vec<&str> = params.split(',').collect();
        let mut count = 0;

        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() && !SELF_PARAM_VARIANTS.iter().any(|s| trimmed.starts_with(s)) {
                count += 1;
            }
        }

        count
    }

    pub(super) fn count_optional_fields(
        lines: &[&str],
        start_line: usize,
        option_pattern: &Regex,
    ) -> usize {
        Self::count_block_matches(lines, start_line, |line| option_pattern.is_match(line))
    }

    pub(super) fn is_trait_fn_declaration(lines: &[&str], start_line: usize) -> bool {
        for line in &lines[start_line..] {
            if line.contains('{') {
                return false;
            }
            if line.trim().ends_with(';') {
                return true;
            }
            if line.contains(';') && !line.contains('{') {
                return true;
            }
        }
        false
    }

    pub(super) fn count_function_lines(lines: &[&str], start_line: usize) -> usize {
        Self::count_block_matches(lines, start_line, |_| true)
    }

    pub(super) fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let Some(path_str) = src_dir.to_str() else {
            return false;
        };
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }

    pub(super) fn should_skip_file(&self, path: &std::path::Path) -> bool {
        let Some(path_str) = path.to_str() else {
            return false;
        };
        self.rules
            .skip_file_patterns
            .iter()
            .any(|pattern| path_str.ends_with(pattern))
    }
}
