use regex::Regex;

use super::KissValidator;

impl KissValidator {
    pub(super) fn count_struct_fields(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_struct = false;
        let mut field_count = 0;
        let field_pattern = match Regex::new(r"^\s*(?:pub\s+)?[a-z_][a-z0-9_]*\s*:") {
            Ok(regex) => regex,
            Err(_) => return 0,
        };

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_struct = true;
            }
            if in_struct {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();
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

    pub(super) fn count_function_params(&self, params: &str) -> usize {
        if params.trim().is_empty() {
            return 0;
        }

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

    pub(super) fn count_optional_fields(
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

    pub(super) fn is_trait_fn_declaration(&self, lines: &[&str], start_line: usize) -> bool {
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

    pub(super) fn count_function_lines(&self, lines: &[&str], start_line: usize) -> usize {
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

    pub(super) fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let path_str = src_dir.to_string_lossy();
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}
