//! `MetricViolation` implementation.

use std::path::PathBuf;

use super::MetricType;
use crate::violation_trait::{Severity, Violation, ViolationCategory};

/// A metric violation when a threshold is exceeded
#[derive(Debug, Clone)]
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

impl std::fmt::Display for MetricViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} `{}` has {} of {} (threshold: {}) in {}",
            self.id(),
            self.metric_type.name(),
            self.item_name,
            self.metric_type.description(),
            self.actual_value,
            self.threshold,
            self.file.display()
        )
    }
}

impl Violation for MetricViolation {
    fn id(&self) -> &str {
        match self.metric_type {
            MetricType::CognitiveComplexity => "METRIC001",
            MetricType::CyclomaticComplexity => "METRIC004",
            MetricType::FunctionLength => "METRIC002",
            MetricType::NestingDepth => "METRIC003",
        }
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
