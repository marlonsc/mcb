//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! AST selector / tree-sitter query rule slice for the declarative validator.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use super::DeclarativeValidator;
use crate::ast::{AstSelectorEngine, TreeSitterQueryExecutor};
use crate::filters::dependency_parser::WorkspaceDependencies;
use crate::filters::rule_filters::RuleFilterExecutor;
use crate::rules::yaml_loader::ValidatedRule;
use mcb_domain::ports::validation::Violation;

impl DeclarativeValidator {
    pub(crate) fn validate_ast_selector_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        let ast_rules: Vec<&ValidatedRule> = rules
            .iter()
            .filter(|r| r.enabled && (!r.selectors.is_empty() || r.ast_query.is_some()))
            .collect();

        if ast_rules.is_empty() {
            return Ok(Vec::new());
        }

        let file_results: Vec<
            mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>>,
        > = files
            .par_iter()
            .map(|file| {
                Self::ast_violations_for_file(file, &ast_rules, filter_executor, workspace_deps)
            })
            .collect();

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();
        for file_result in file_results {
            violations.extend(file_result?);
        }

        Ok(violations)
    }

    /// Collect AST selector and tree-sitter query violations for a single file.
    fn ast_violations_for_file(
        file: &Path,
        ast_rules: &[&ValidatedRule],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> mcb_domain::ports::validation::ValidatorResult<Vec<Box<dyn Violation>>> {
        let Some(source) = super::read_cached_or_disk(file, "AST validation") else {
            return Ok(Vec::new());
        };

        let mut local: Vec<Box<dyn Violation>> = Vec::new();
        for rule in ast_rules {
            if !Self::should_execute_on_file(
                filter_executor,
                workspace_deps,
                rule,
                file,
                Some(source.as_ref()),
            ) {
                continue;
            }

            let selector_matches =
                AstSelectorEngine::execute_on_source(rule, file, source.as_ref());
            local.extend(selector_matches.into_iter().map(|matched| {
                Self::build_pattern_match_violation(rule, matched.file_path, matched.line)
            }));

            let query_matches =
                TreeSitterQueryExecutor::execute_on_source(rule, file, source.as_bytes())?;
            local.extend(query_matches.into_iter().map(|matched| {
                Self::build_pattern_match_violation(rule, matched.file_path, matched.line)
            }));
        }

        Ok(local)
    }
}
