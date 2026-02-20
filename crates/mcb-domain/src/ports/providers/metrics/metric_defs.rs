#![allow(missing_docs)]

use std::collections::HashMap;

pub type MetricLabels = HashMap<String, String>;
pub type MetricsResult<T> = crate::Result<T>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetricsError {
    #[error("Metric not found: {name}")]
    NotFound { name: String },
    #[error("Invalid metric: {message}")]
    Invalid { message: String },
    #[error("Metrics backend error: {message}")]
    Backend { message: String },
}

pub(crate) fn labels_from<const N: usize>(pairs: [(&str, &str); N]) -> MetricLabels {
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}
