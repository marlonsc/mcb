//! External Provider Ports
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

#![allow(missing_docs)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use derive_more::Display;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::entities::CodeChunk;
use crate::entities::project::ProjectType;
use crate::entities::vcs::{RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository};
use crate::error::Result;
use crate::ports::{RuleInfo, ValidationReport};
use crate::value_objects::{
    CollectionId, CollectionInfo, Embedding, FileInfo, Language, SearchResult,
};
use crate::value_objects::{EmbeddingConfig, VectorStoreConfig};

// ============================================================================
// Analysis
// ============================================================================

/// Complexity finding for a single function.
#[derive(Debug, Clone)]
pub struct ComplexityFinding {
    pub file: PathBuf,
    pub function: String,
    pub complexity: u32,
}

/// Dead code finding for a single symbol.
#[derive(Debug, Clone)]
pub struct DeadCodeFinding {
    pub file: PathBuf,
    pub line: usize,
    pub item_type: String,
    pub name: String,
}

/// TDG finding for a single file.
#[derive(Debug, Clone)]
pub struct TdgFinding {
    pub file: PathBuf,
    pub score: u32,
}

/// Complexity analysis provider.
pub trait ComplexityAnalyzer: Send + Sync {
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<ComplexityFinding>>;
}

/// Dead code detection provider.
pub trait DeadCodeDetector: Send + Sync {
    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<DeadCodeFinding>>;
}

/// Technical Debt Gradient scoring provider.
pub trait TdgScorer: Send + Sync {
    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<TdgFinding>>;
}

// ============================================================================
// Config
// ============================================================================

/// Provider configuration manager interface
#[async_trait::async_trait]
pub trait ProviderConfigManagerInterface: Send + Sync {
    fn get_embedding_config(&self, name: &str) -> Result<&EmbeddingConfig>;
    fn get_vector_store_config(&self, name: &str) -> Result<&VectorStoreConfig>;
    fn list_embedding_providers(&self) -> Vec<String>;
    fn list_vector_store_providers(&self) -> Vec<String>;

    fn has_embedding_provider(&self, name: &str) -> bool {
        self.list_embedding_providers().contains(&name.to_owned())
    }

    fn has_vector_store_provider(&self, name: &str) -> bool {
        self.list_vector_store_providers()
            .contains(&name.to_owned())
    }

    fn get_default_embedding_config(&self) -> Option<&EmbeddingConfig> {
        let providers = self.list_embedding_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_embedding_config(&providers[0]).ok()
        }
    }

    fn get_default_vector_store_config(&self) -> Option<&VectorStoreConfig> {
        let providers = self.list_vector_store_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_vector_store_config(&providers[0]).ok()
        }
    }
}

// ============================================================================
// Crypto
// ============================================================================

/// Encrypted data container
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
#[display(
    "EncryptedData {{ ciphertext: {} bytes, nonce: {} bytes }}",
    ciphertext.len(),
    nonce.len()
)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    #[must_use]
    pub fn new(ciphertext: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { ciphertext, nonce }
    }
}

/// Cryptographic provider port
#[async_trait]
pub trait CryptoProvider: Send + Sync {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData>;
    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>>;
    fn provider_name(&self) -> &str;
}

// ============================================================================
// Embedding
// ============================================================================

/// AI Semantic Understanding Interface
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_owned()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::embedding("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;

    async fn health_check(&self) -> Result<()> {
        self.embed("health check").await?;
        Ok(())
    }
}

// ============================================================================
// HTTP
// ============================================================================

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    pub max_idle_per_host: usize,
    pub idle_timeout: Duration,
    pub keepalive: Duration,
    pub timeout: Duration,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keepalive: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            user_agent: "mcb/domain-client".to_owned(),
        }
    }
}

/// HTTP client provider trait
pub trait HttpClientProvider: Send + Sync {
    fn client(&self) -> &Client;
    fn config(&self) -> &HttpClientConfig;
    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> std::result::Result<Client, Box<dyn std::error::Error + Send + Sync>>;
    fn is_enabled(&self) -> bool;
}

// ============================================================================
// Hybrid Search
// ============================================================================

/// Result of a hybrid search operation
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub result: SearchResult,
    pub bm25_score: f32,
    pub semantic_score: f32,
    pub hybrid_score: f32,
}

/// Port for hybrid search operations
#[async_trait]
pub trait HybridSearchProvider: Send + Sync {
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    async fn search(
        &self,
        collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    async fn clear_collection(&self, collection: &str) -> Result<()>;
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}

// ============================================================================
// Language Chunking
// ============================================================================

/// Language-Specific Code Chunking Provider
pub trait LanguageChunkingProvider: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&'static str];
    fn chunk(&self, content: &str, file_path: &str) -> Vec<CodeChunk>;
    fn provider_name(&self) -> &str;

    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }

    fn max_chunk_size(&self) -> usize {
        50
    }
}

// ============================================================================
// Metrics
// ============================================================================

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

#[async_trait]
pub trait MetricsProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn increment(&self, name: &str, labels: &MetricLabels) -> MetricsResult<()>;
    async fn increment_by(
        &self,
        name: &str,
        value: f64,
        labels: &MetricLabels,
    ) -> MetricsResult<()>;
    async fn gauge(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;
    async fn histogram(&self, name: &str, value: f64, labels: &MetricLabels) -> MetricsResult<()>;

    async fn record_index_time(&self, duration: Duration, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.histogram(
            "mcb_index_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    async fn record_search_latency(
        &self,
        duration: Duration,
        collection: &str,
    ) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.histogram(
            "mcb_search_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    async fn record_embedding_latency(
        &self,
        duration: Duration,
        provider: &str,
    ) -> MetricsResult<()> {
        let labels = labels_from([("provider", provider)]);
        self.histogram(
            "mcb_embedding_duration_seconds",
            duration.as_secs_f64(),
            &labels,
        )
        .await
    }

    async fn increment_indexed_files(&self, collection: &str, count: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment_by("mcb_indexed_files_total", count as f64, &labels)
            .await
    }

    async fn increment_search_requests(&self, collection: &str) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.increment("mcb_search_requests_total", &labels).await
    }

    async fn set_active_indexing_jobs(&self, count: u64) -> MetricsResult<()> {
        self.gauge("mcb_active_indexing_jobs", count as f64, &HashMap::new())
            .await
    }

    async fn set_vector_store_size(&self, collection: &str, vectors: u64) -> MetricsResult<()> {
        let labels = labels_from([("collection", collection)]);
        self.gauge("mcb_vector_store_size", vectors as f64, &labels)
            .await
    }

    async fn record_cache_access(&self, hit: bool, cache_type: &str) -> MetricsResult<()> {
        let labels = labels_from([
            ("cache_type", cache_type),
            ("result", if hit { "hit" } else { "miss" }),
        ]);
        self.increment("mcb_cache_accesses_total", &labels).await
    }
}

// ============================================================================
// Project Detection
// ============================================================================

/// Configuration for project detector initialization
#[derive(Debug, Clone)]
pub struct ProjectDetectorConfig {
    pub repo_path: String,
}

/// Registry entry for project detectors
pub struct ProjectDetectorEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub marker_files: &'static [&'static str],
    pub build: fn(&ProjectDetectorConfig) -> Result<Arc<dyn ProjectDetector>>,
}

/// Project detector trait
#[async_trait]
pub trait ProjectDetector: Send + Sync {
    async fn detect(&self, path: &Path) -> Result<Option<ProjectType>>;
    fn marker_files(&self) -> &[&str];
    fn detector_name(&self) -> &str;
}

#[linkme::distributed_slice]
pub static PROJECT_DETECTORS: [ProjectDetectorEntry] = [..];

// ============================================================================
// Validation
// ============================================================================

/// Information about a validator available in a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_count: usize,
    pub categories: Vec<String>,
}

/// Options for validation operations
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    pub validators: Option<Vec<String>>,
    pub severity_filter: Option<String>,
    pub exclude_patterns: Option<Vec<String>>,
    pub max_files: Option<usize>,
    pub include_suggestions: bool,
}

/// Request for running one or more rule validators.
#[derive(Debug, Clone)]
pub struct RuleValidatorRequest {
    pub workspace_root: std::path::PathBuf,
    pub validator_names: Option<Vec<String>>,
    pub severity_filter: Option<String>,
    pub exclude_patterns: Option<Vec<String>>,
}

/// Port for a single rule validator
pub trait RuleValidator: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(
        &self,
        request: &RuleValidatorRequest,
    ) -> std::result::Result<ValidationReport, crate::error::Error>;
}

/// Pluggable Validation Provider
#[async_trait]
pub trait ValidationProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn description(&self) -> &str;
    fn list_validators(&self) -> Vec<ValidatorInfo>;
    fn get_rules(&self, category: Option<&str>) -> Vec<RuleInfo>;
    async fn validate(
        &self,
        workspace_root: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport>;
    async fn validate_file(
        &self,
        file_path: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport>;
    fn can_validate(&self, path: &Path) -> bool;
    fn supported_extensions(&self) -> &[&str];
}

// ============================================================================
// Version Control (VCS)
// ============================================================================

/// Version Control System provider for repository operations.
#[async_trait]
pub trait VcsProvider: Send + Sync {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository>;
    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId;
    async fn list_branches(&self, repo: &VcsRepository) -> Result<Vec<VcsBranch>>;
    async fn commit_history(
        &self,
        repo: &VcsRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>>;
    async fn list_files(&self, repo: &VcsRepository, branch: &str) -> Result<Vec<PathBuf>>;
    async fn read_file(&self, repo: &VcsRepository, branch: &str, path: &Path) -> Result<String>;
    fn vcs_name(&self) -> &str;
    async fn diff_refs(
        &self,
        repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff>;
    async fn list_repositories(&self, root: &Path) -> Result<Vec<VcsRepository>>;
}

// ============================================================================
// Vector Store
// ============================================================================

#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool>;
    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>>;
    async fn flush(&self, collection: &CollectionId) -> Result<()>;
    fn provider_name(&self) -> &str;

    async fn health_check(&self) -> Result<()> {
        let health_check_id = CollectionId::from_name("__health_check__");
        self.collection_exists(&health_check_id).await?;
        Ok(())
    }
}

#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;
    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>>;
    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>>;
}

#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + VectorStoreBrowser + Send + Sync {
    async fn create_collection(&self, collection: &CollectionId, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, collection: &CollectionId) -> Result<()>;
    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>>;
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;
    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()>;
    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;
    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}
