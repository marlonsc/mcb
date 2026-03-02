//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! `MetricViolation` implementation.

use std::path::PathBuf;

use derive_more::Display;

use super::MetricType;
use mcb_domain::ports::validation::{Severity, Violation, ViolationCategory};

/// A metric violation when a threshold is exceeded
#[derive(Debug, Clone, Display)]
#[display(
    "[{}] {} `{}` has {} of {} (threshold: {}) in {}",
    self.metric_rule_id(),
    metric_type.name(),
    item_name,
    metric_type.description(),
    actual_value,
    threshold,
    file.display()
)]
pub struct MetricViolation {
    /// File path
    pub file: PathBuf,
    /// Line number where the function/item starts
    pub line: usize,
    /// Name of the function/item
    pub item_name: String,
    /// Type of metric that was exceeded
    pub metric_type: MetricType,
    /// Actual value measured
    pub actual_value: u32,
    /// Configured threshold
    pub threshold: u32,
    /// Severity level
    pub severity: Severity,
}

impl MetricViolation {
    fn metric_rule_id(&self) -> &'static str {
        match self.metric_type {
            MetricType::CognitiveComplexity => "METRIC001",
            MetricType::CyclomaticComplexity => "METRIC004",
            MetricType::FunctionLength => "METRIC002",
            MetricType::NestingDepth => "METRIC003",
        }
    }
}

impl Violation for MetricViolation {
    fn id(&self) -> &str {
        self.metric_rule_id()
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&PathBuf> {
        Some(&self.file)
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn message(&self) -> String {
        format!(
            "{} `{}` has {} of {} (threshold: {})",
            self.metric_type.name(),
            self.item_name,
            self.metric_type.description(),
            self.actual_value,
            self.threshold
        )
    }

    fn suggestion(&self) -> Option<String> {
        Some(self.metric_type.suggestion())
    }
}
