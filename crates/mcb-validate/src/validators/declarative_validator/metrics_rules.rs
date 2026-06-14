//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Metrics rule slice for the declarative validator.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use super::DeclarativeValidator;
use crate::metrics::{MetricThresholds, MetricViolation, RcaAnalyzer};
use crate::rules::yaml_loader::ValidatedRule;
use mcb_domain::ports::validation::Violation;

impl DeclarativeValidator {
    pub(crate) fn validate_metrics_rules(
        rules: &[ValidatedRule],
        files: &[PathBuf],
    ) -> Vec<Box<dyn Violation>> {
        let metrics_rules: Vec<(&ValidatedRule, MetricThresholds)> = rules
            .iter()
            .filter(|r| r.enabled && r.metrics.is_some())
            .filter_map(|rule| {
                rule.metrics
                    .as_ref()
                    .map(|cfg| (rule, MetricThresholds::from_metrics_config(cfg)))
            })
            .collect();

        if metrics_rules.is_empty() {
            return Vec::new();
        }

        let analyzer = RcaAnalyzer::new();
        let per_file: Vec<Vec<Box<dyn Violation>>> = files
            .par_iter()
            .map(|file| Self::metrics_violations_for_file(&analyzer, file, &metrics_rules))
            .collect();

        per_file.into_iter().flatten().collect()
    }

    /// Collect metric violations for a single file across all metrics rules.
    fn metrics_violations_for_file(
        analyzer: &RcaAnalyzer,
        file: &Path,
        metrics_rules: &[(&ValidatedRule, MetricThresholds)],
    ) -> Vec<Box<dyn Violation>> {
        let functions = match analyzer.analyze_file(file) {
            Ok(functions) => functions,
            Err(e) => {
                mcb_domain::warn!(
                    "validate",
                    "Metrics analysis failed",
                    &format!("file = {}, error = {:?}", file.display(), e)
                );
                return Vec::new();
            }
        };

        let mut local: Vec<Box<dyn Violation>> = Vec::new();
        for (rule, thresholds) in metrics_rules {
            mcb_domain::trace!(
                "declarative",
                "Metrics check",
                &format!("rule={} file={}", rule.id, file.display())
            );
            let rule_violations: Vec<MetricViolation> =
                RcaAnalyzer::find_violations_in_functions(file, &functions, thresholds);
            local.extend(
                rule_violations
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn Violation>),
            );
        }
        local
    }
}
