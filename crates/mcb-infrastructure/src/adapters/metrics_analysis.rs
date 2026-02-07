//! Metrics Analysis Adapter
//!
//! Implementation of mapping between domain and real metrics analysis providers.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::metrics_analysis::{
    FileMetrics, FunctionMetrics, MetricsAnalysisProvider,
};
use mcb_domain::value_objects::SupportedLanguage;
use std::path::Path;

/// Null metrics analysis provider for testing or fallback
pub struct NullMetricsProvider;

impl NullMetricsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricsAnalysisProvider for NullMetricsProvider {
    fn provider_name(&self) -> &str {
        "null"
    }

    fn supported_languages(&self) -> &[SupportedLanguage] {
        &[]
    }

    async fn analyze_file(&self, path: &Path) -> Result<FileMetrics> {
        Ok(FileMetrics {
            file: path.to_string_lossy().to_string(),
            ..Default::default()
        })
    }

    async fn analyze_code(
        &self,
        _content: &[u8],
        _language: SupportedLanguage,
        file_path: Option<&str>,
    ) -> Result<FileMetrics> {
        Ok(FileMetrics {
            file: file_path.unwrap_or("").to_string(),
            ..Default::default()
        })
    }

    async fn analyze_functions(&self, _path: &Path) -> Result<Vec<FunctionMetrics>> {
        Ok(Vec::new())
    }
}
