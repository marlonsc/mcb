//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use std::path::Path;

use async_trait::async_trait;

use crate::error::Result;
use crate::value_objects::SupportedLanguage;

use super::types::{FileMetrics, FunctionMetrics};

#[async_trait]
pub trait MetricsAnalysisProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_languages(&self) -> &[SupportedLanguage];

    fn supports_language(&self, lang: SupportedLanguage) -> bool {
        self.supported_languages().contains(&lang)
    }

    async fn analyze_file(&self, path: &Path) -> Result<FileMetrics>;

    async fn analyze_code(
        &self,
        content: &[u8],
        language: SupportedLanguage,
        file_path: Option<&str>,
    ) -> Result<FileMetrics>;

    async fn analyze_functions(&self, path: &Path) -> Result<Vec<FunctionMetrics>>;

    async fn analyze_function(
        &self,
        path: &Path,
        function_name: &str,
    ) -> Result<Option<FunctionMetrics>> {
        let functions = self.analyze_functions(path).await?;
        Ok(functions.into_iter().find(|f| f.name == function_name))
    }

    fn can_analyze(&self, path: &Path) -> bool {
        SupportedLanguage::from_path(path).is_some_and(|lang| self.supports_language(lang))
    }
}
