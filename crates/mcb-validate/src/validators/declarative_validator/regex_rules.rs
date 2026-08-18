//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Regex pattern rule slice for the declarative validator.

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use regex::Regex;

use super::DeclarativeValidator;
use crate::filters::dependency_parser::WorkspaceDependencies;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::rules::yaml_loader::ValidatedRule;
use mcb_domain::ports::validation::Violation;
use mcb_utils::utils::regex::compile_regex;

struct CompiledRegexRule<'a> {
    rule: &'a ValidatedRule,
    compiled: Vec<Regex>,
    ignore_compiled: Vec<Regex>,
}

impl DeclarativeValidator {
    pub(crate) fn validate_regex_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> Vec<Box<dyn Violation>> {
        let regex_rules: Vec<CompiledRegexRule<'_>> = rules
            .iter()
            .filter(|r| Self::is_regex_rule(r))
            .filter_map(|rule| {
                let compiled = Self::compile_rule_patterns(rule);
                if compiled.is_empty() {
                    return None;
                }

                Some(CompiledRegexRule {
                    rule,
                    compiled,
                    ignore_compiled: Self::compile_ignore_patterns(rule),
                })
            })
            .collect();

        if regex_rules.is_empty() {
            return Vec::new();
        }

        let per_file: Vec<Vec<Box<dyn Violation>>> = files
            .par_iter()
            .map(|file| {
                Self::regex_violations_for_file(file, &regex_rules, filter_executor, workspace_deps)
            })
            .collect();

        per_file.into_iter().flatten().collect()
    }

    /// Collect regex-rule violations for a single file.
    fn regex_violations_for_file(
        file: &Path,
        regex_rules: &[CompiledRegexRule<'_>],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> Vec<Box<dyn Violation>> {
        let Some(content) = super::read_cached_or_disk(file, "regex validation") else {
            return Vec::new();
        };

        let mut file_violations: Vec<Box<dyn Violation>> = Vec::new();
        for rule in regex_rules {
            if Self::should_execute_on_file(
                filter_executor,
                workspace_deps,
                rule.rule,
                file,
                Some(content.as_ref()),
            ) {
                file_violations.extend(Self::collect_regex_violations_for_content(
                    rule.rule,
                    file,
                    content.as_ref(),
                    &rule.compiled,
                    &rule.ignore_compiled,
                ));
            }
        }
        file_violations
    }

    fn is_regex_rule(rule: &ValidatedRule) -> bool {
        let uses_regex_engine = rule.lint_select.is_empty()
            && rule.metrics.is_none()
            && rule.selectors.is_empty()
            && rule.ast_query.is_none();
        rule.enabled && uses_regex_engine && Self::has_rule_patterns(rule)
    }

    fn has_rule_patterns(rule: &ValidatedRule) -> bool {
        Self::effective_regex_config(rule)
            .get("patterns")
            .and_then(|v| v.as_object())
            .is_some()
    }

    fn effective_regex_config(rule: &ValidatedRule) -> &serde_json::Value {
        let top_level_has_patterns = rule
            .config
            .get("patterns")
            .and_then(|v| v.as_object())
            .is_some();
        let top_level_has_ignore = rule
            .config
            .get("ignore_patterns")
            .and_then(|v| v.as_array())
            .is_some();

        if top_level_has_patterns || top_level_has_ignore {
            return &rule.config;
        }

        rule.rule_definition.get("config").unwrap_or(&rule.config)
    }

    fn compile_rule_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(patterns_obj) = Self::effective_regex_config(rule)
            .get("patterns")
            .and_then(|v| v.as_object())
        else {
            return Vec::new();
        };

        patterns_obj
            .iter()
            .filter_map(|(name, val)| {
                let pat = val.as_str()?;
                match compile_regex(pat) {
                    Ok(rx) => Some(rx),
                    Err(e) => {
                        mcb_domain::warn!(
                            "validate",
                            "Malformed regex pattern in rule",
                            &format!(
                                "rule_id = {}, pattern_name = {}, error = {:?}",
                                rule.id, name, e
                            )
                        );
                        None
                    }
                }
            })
            .collect()
    }

    fn compile_ignore_patterns(rule: &ValidatedRule) -> Vec<Regex> {
        let Some(ignore_arr) = Self::effective_regex_config(rule)
            .get("ignore_patterns")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };

        ignore_arr
            .iter()
            .filter_map(|v| {
                let pat = v.as_str()?;
                match compile_regex(pat) {
                    Ok(rx) => Some(rx),
                    Err(_) => {
                        mcb_domain::warn!("validate", "Invalid ignore pattern regex", &rule.id);
                        None
                    }
                }
            })
            .collect()
    }

    fn collect_regex_violations_for_content(
        rule: &ValidatedRule,
        file: &Path,
        content: &str,
        compiled: &[Regex],
        ignore_compiled: &[Regex],
    ) -> Vec<Box<dyn Violation>> {
        content
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                if ignore_compiled.iter().any(|irx| irx.is_match(line)) {
                    return None;
                }
                if !compiled.iter().any(|rx| rx.is_match(line)) {
                    return None;
                }

                Some(Self::build_pattern_match_violation(
                    rule,
                    file.to_path_buf(),
                    line_num + 1,
                ))
            })
            .collect()
    }
}
