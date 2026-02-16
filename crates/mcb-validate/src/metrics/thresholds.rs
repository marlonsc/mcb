//! Metric thresholds configuration
//!
//! Defines threshold values for various code metrics and how they map to violations.

use std::collections::HashMap;

use crate::constants::rules::{METRICS_FIELD_MAX, METRICS_FIELD_SEVERITY};
use crate::constants::severities::{SEVERITY_ERROR, SEVERITY_INFO};
use crate::traits::violation::Severity;

/// Types of metrics we can measure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Cognitive complexity - how hard code is to understand
    CognitiveComplexity,
    /// Cyclomatic complexity - number of linearly independent paths
    CyclomaticComplexity,
    /// Number of lines/statements in a function
    FunctionLength,
    /// Maximum nesting depth (if/for/while/match)
    NestingDepth,
}

impl MetricType {
    /// Get the human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::CognitiveComplexity
            | Self::CyclomaticComplexity
            | Self::FunctionLength
            | Self::NestingDepth => "Function",
        }
    }

    /// Get metric description
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::CognitiveComplexity => "cognitive complexity",
            Self::CyclomaticComplexity => "cyclomatic complexity",
            Self::FunctionLength => "length",
            Self::NestingDepth => "nesting depth",
        }
    }

    /// Get suggestion for fixing
    #[must_use]
    pub fn suggestion(&self) -> String {
        match self {
            Self::CognitiveComplexity => {
                "Consider breaking this function into smaller, focused functions. \
                 Extract complex conditions into named functions or early returns."
                    .to_owned()
            }
            Self::CyclomaticComplexity => {
                "Reduce the number of decision points (if/else, switch, loops). \
                 Consider using polymorphism or strategy pattern instead of conditionals."
                    .to_owned()
            }
            Self::FunctionLength => {
                "Consider extracting helper functions or using the Extract Method refactoring. \
                 Functions should ideally do one thing well."
                    .to_owned()
            }
            Self::NestingDepth => {
                "Consider using early returns, guard clauses, or extracting nested logic \
                 into separate functions to reduce nesting."
                    .to_owned()
            }
        }
    }
}

/// A single metric threshold configuration
#[derive(Debug, Clone)]
pub struct MetricThreshold {
    /// Maximum allowed value
    pub max_value: u32,
    /// Severity when threshold is exceeded
    pub severity: Severity,
}

/// Configuration for all metric thresholds
#[derive(Debug, Clone)]
pub struct MetricThresholds {
    thresholds: HashMap<MetricType, MetricThreshold>,
}

impl Default for MetricThresholds {
    fn default() -> Self {
        Self::new()
            // Default thresholds based on common industry standards
            .with_threshold(MetricType::CognitiveComplexity, 15, Severity::Warning)
            .with_threshold(MetricType::CyclomaticComplexity, 10, Severity::Warning)
            .with_threshold(MetricType::FunctionLength, 50, Severity::Warning)
            .with_threshold(MetricType::NestingDepth, 4, Severity::Warning)
    }
}

impl MetricThresholds {
    /// Create empty thresholds
    #[must_use]
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
        }
    }

    /// Add or update a threshold
    #[must_use]
    pub fn with_threshold(
        mut self,
        metric: MetricType,
        max_value: u32,
        severity: Severity,
    ) -> Self {
        self.thresholds.insert(
            metric,
            MetricThreshold {
                max_value,
                severity,
            },
        );
        self
    }

    /// Get threshold for a metric type
    #[must_use]
    pub fn get(&self, metric: MetricType) -> Option<&MetricThreshold> {
        self.thresholds.get(&metric)
    }

    fn severity_from_str(s: Option<&str>) -> Severity {
        match s {
            Some(SEVERITY_ERROR) => Severity::Error,
            Some(SEVERITY_INFO) => Severity::Info,
            _ => Severity::Warning,
        }
    }

    fn to_u32(val: u64) -> u32 {
        u32::try_from(val).unwrap_or(u32::MAX)
    }

    fn parse_metric(
        obj: &serde_json::Map<String, serde_json::Value>,
        key: &str,
    ) -> Option<(u32, Severity)> {
        let section = obj.get(key)?;
        let max = section.get(METRICS_FIELD_MAX)?.as_u64()?;
        let severity_str = section.get(METRICS_FIELD_SEVERITY).and_then(|v| v.as_str());
        Some((Self::to_u32(max), Self::severity_from_str(severity_str)))
    }

    fn ensure_defaults(mut self) -> Self {
        const DEFAULTS: [(MetricType, u32, Severity); 4] = [
            (MetricType::CognitiveComplexity, 15, Severity::Warning),
            (MetricType::CyclomaticComplexity, 10, Severity::Warning),
            (MetricType::FunctionLength, 50, Severity::Warning),
            (MetricType::NestingDepth, 4, Severity::Warning),
        ];

        for (metric, max, severity) in &DEFAULTS {
            if self.get(*metric).is_none() {
                self = self.with_threshold(*metric, *max, *severity);
            }
        }
        self
    }

    /// Parse thresholds from YAML rule config
    #[must_use]
    pub fn from_yaml(config: &serde_json::Value) -> Self {
        let mut thresholds = Self::new();

        if let Some(obj) = config.as_object() {
            if let Some((max, sev)) = Self::parse_metric(obj, "cognitive_complexity") {
                thresholds = thresholds.with_threshold(MetricType::CognitiveComplexity, max, sev);
            }
            if let Some((max, sev)) = Self::parse_metric(obj, "function_length") {
                thresholds = thresholds.with_threshold(MetricType::FunctionLength, max, sev);
            }
            if let Some((max, sev)) = Self::parse_metric(obj, "cyclomatic_complexity") {
                thresholds = thresholds.with_threshold(MetricType::CyclomaticComplexity, max, sev);
            }
            if let Some((max, sev)) = Self::parse_metric(obj, "nesting_depth") {
                thresholds = thresholds.with_threshold(MetricType::NestingDepth, max, sev);
            }
        }

        thresholds.ensure_defaults()
    }

    /// Create thresholds from a `MetricsConfig` struct (from `ValidatedRule`)
    #[must_use]
    pub fn from_metrics_config(config: &crate::rules::yaml_loader::MetricsConfig) -> Self {
        let mut thresholds = Self::new();

        if let Some(cc) = &config.cognitive_complexity {
            let sev = Self::severity_from_str(cc.severity.as_deref());
            thresholds = thresholds.with_threshold(MetricType::CognitiveComplexity, cc.max, sev);
        }

        if let Some(cyc) = &config.cyclomatic_complexity {
            let sev = Self::severity_from_str(cyc.severity.as_deref());
            if config.cognitive_complexity.is_none() {
                thresholds =
                    thresholds.with_threshold(MetricType::CognitiveComplexity, cyc.max, sev);
            }
        }

        if let Some(fl) = &config.function_length {
            let sev = Self::severity_from_str(fl.severity.as_deref());
            thresholds = thresholds.with_threshold(MetricType::FunctionLength, fl.max, sev);
        }

        if let Some(nd) = &config.nesting_depth {
            let sev = Self::severity_from_str(nd.severity.as_deref());
            thresholds = thresholds.with_threshold(MetricType::NestingDepth, nd.max, sev);
        }

        thresholds
    }
}
