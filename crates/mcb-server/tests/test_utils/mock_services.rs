//! Mock implementations of domain service interfaces for testing
#![allow(dead_code, unused_mut)]

use async_trait::async_trait;
use mcb_application::domain_services::search::{
    ContextServiceInterface, IndexingResult, IndexingServiceInterface, IndexingStatus,
    SearchServiceInterface,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::entities::vcs::{
    DiffStatus, FileDiff, RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::VcsProvider;
use mcb_domain::value_objects::{Embedding, SearchResult};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Search Service
// ============================================================================

/// Mock implementation of SearchServiceInterface for testing
pub struct MockSearchService {
    /// Pre-configured results to return
    results: Arc<Mutex<Vec<SearchResult>>>,
    /// Whether the next call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockSearchService {
    /// Create a new mock search service
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated search failure".to_string())),
        }
    }

    /// Configure the mock to return specific results
    pub fn with_results(self, results: Vec<SearchResult>) -> Self {
        *self.results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchServiceInterface for MockSearchService {
    async fn search(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }

    async fn search_with_filters(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
        _filters: Option<&mcb_application::ports::services::SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        // Mock ignores filters and delegates to search
        self.search(collection, query, limit).await
    }
}

// ============================================================================
// Mock Indexing Service
// ============================================================================

/// Mock implementation of IndexingServiceInterface for testing
pub struct MockIndexingService {
    /// Pre-configured indexing result
    indexing_result: Arc<Mutex<Option<IndexingResult>>>,
    /// Current status to return
    status: Arc<Mutex<IndexingStatus>>,
    /// Whether the next indexing call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockIndexingService {
    /// Create a new mock indexing service
    pub fn new() -> Self {
        Self {
            indexing_result: Arc::new(Mutex::new(Some(IndexingResult {
                files_processed: 0,
                chunks_created: 0,
                files_skipped: 0,
                errors: Vec::new(),
                operation_id: None,
                status: "completed".to_string(),
            }))),
            status: Arc::new(Mutex::new(IndexingStatus::default())),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated indexing failure".to_string())),
        }
    }

    /// Configure the mock to return specific indexing result
    pub fn with_result(self, result: IndexingResult) -> Self {
        *self.indexing_result.lock().expect("Lock poisoned") = Some(result);
        self
    }

    /// Configure the mock to return specific status
    pub fn with_status(self, status: IndexingStatus) -> Self {
        *self.status.lock().expect("Lock poisoned") = status;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockIndexingService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexingServiceInterface for MockIndexingService {
    async fn index_codebase(&self, _path: &Path, _collection: &str) -> Result<IndexingResult> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let result = self.indexing_result.lock().expect("Lock poisoned");
        Ok(result.clone().unwrap_or_else(|| IndexingResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
            operation_id: None,
            status: "completed".to_string(),
        }))
    }

    fn get_status(&self) -> IndexingStatus {
        self.status.lock().expect("Lock poisoned").clone()
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }
}

// ============================================================================
// Mock Context Service
// ============================================================================

/// Mock implementation of ContextServiceInterface for testing
pub struct MockContextService {
    /// Pre-configured search results
    search_results: Arc<Mutex<Vec<SearchResult>>>,
    /// Embedding dimensions
    dimensions: usize,
    /// Whether the next call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockContextService {
    /// Create a new mock context service
    pub fn new() -> Self {
        Self {
            search_results: Arc::new(Mutex::new(Vec::new())),
            dimensions: 384,
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated context failure".to_string())),
        }
    }

    /// Configure the mock to return specific search results
    pub fn with_search_results(self, results: Vec<SearchResult>) -> Self {
        *self.search_results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to use specific dimensions
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.dimensions = dims;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockContextService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextServiceInterface for MockContextService {
    async fn initialize(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn store_chunks(&self, _collection: &str, _chunks: &[CodeChunk]) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.search_results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }

    async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(Embedding {
            vector: vec![0.1; self.dimensions],
            model: "mock".to_string(),
            dimensions: self.dimensions,
        })
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        // Return (total_chunks, total_queries)
        Ok((100, 10))
    }

    fn embedding_dimensions(&self) -> usize {
        self.dimensions
    }
}

// ============================================================================
// Mock Validation Service
// ============================================================================

use mcb_application::ports::services::{
    ValidationReport, ValidationServiceInterface, ViolationEntry,
};

/// Mock implementation of ValidationServiceInterface for testing
pub struct MockValidationService {
    /// Pre-configured validation report
    report: Arc<Mutex<ValidationReport>>,
    /// List of available validators
    validators: Arc<Mutex<Vec<String>>>,
    /// Whether the next call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockValidationService {
    /// Create a new mock validation service
    pub fn new() -> Self {
        Self {
            report: Arc::new(Mutex::new(ValidationReport {
                total_violations: 0,
                errors: 0,
                warnings: 0,
                infos: 0,
                violations: Vec::new(),
                passed: true,
            })),
            validators: Arc::new(Mutex::new(vec![
                "clean_architecture".into(),
                "solid".into(),
                "quality".into(),
            ])),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated validation failure".to_string())),
        }
    }

    /// Configure the mock to return specific validation report
    pub fn with_report(self, report: ValidationReport) -> Self {
        *self.report.lock().expect("Lock poisoned") = report;
        self
    }

    /// Configure the mock to return specific validators
    pub fn with_validators(self, validators: Vec<String>) -> Self {
        *self.validators.lock().expect("Lock poisoned") = validators;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }

    /// Create a mock with violations for testing
    pub fn with_violations(violations: Vec<ViolationEntry>) -> Self {
        let errors = violations.iter().filter(|v| v.severity == "ERROR").count();
        let warnings = violations
            .iter()
            .filter(|v| v.severity == "WARNING")
            .count();
        let infos = violations.iter().filter(|v| v.severity == "INFO").count();

        Self::new().with_report(ValidationReport {
            total_violations: violations.len(),
            errors,
            warnings,
            infos,
            violations,
            passed: errors == 0,
        })
    }
}

impl Default for MockValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for MockValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(self.report.lock().expect("Lock poisoned").clone())
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(self.validators.lock().expect("Lock poisoned").clone())
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(self.report.lock().expect("Lock poisoned").clone())
    }

    async fn get_rules(
        &self,
        _category: Option<&str>,
    ) -> Result<Vec<mcb_application::ports::services::RuleInfo>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        _include_functions: bool,
    ) -> Result<mcb_application::ports::services::ComplexityReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(mcb_application::ports::services::ComplexityReport {
            file: file_path.to_string_lossy().to_string(),
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            functions: Vec::new(),
        })
    }
}

// ============================================================================
// Mock Memory Repository
// ============================================================================

use mcb_domain::entities::memory::{
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationType,
    SessionSummary,
};
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::ports::repositories::memory_repository::FtsSearchResult;

pub struct MockMemoryRepository {
    observations: Arc<Mutex<Vec<Observation>>>,
    summaries: Arc<Mutex<Vec<SessionSummary>>>,
}

impl MockMemoryRepository {
    pub fn new() -> Self {
        Self {
            observations: Arc::new(Mutex::new(Vec::new())),
            summaries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryRepository for MockMemoryRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        self.observations
            .lock()
            .expect("Lock poisoned")
            .push(observation.clone());
        Ok(())
    }

    async fn get_observation(&self, id: &str) -> Result<Option<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs.iter().find(|o| o.id == id).cloned())
    }

    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs.iter().find(|o| o.content_hash == content_hash).cloned())
    }

    async fn search(
        &self,
        _query_embedding: &[f32],
        filter: MemoryFilter,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        let results: Vec<MemorySearchResult> = obs
            .iter()
            .filter(|o| {
                if let Some(ref session) = filter.session_id
                    && o.metadata.session_id.as_ref() != Some(session)
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|o| MemorySearchResult {
                id: o.id.clone(),
                observation: o.clone(),
                similarity_score: 0.9,
            })
            .collect();
        Ok(results)
    }

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        self.summaries
            .lock()
            .expect("Lock poisoned")
            .push(summary.clone());
        Ok(())
    }

    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>> {
        let summaries = self.summaries.lock().expect("Lock poisoned");
        Ok(summaries
            .iter()
            .find(|s| s.session_id == session_id)
            .cloned())
    }

    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        let anchor_idx = obs.iter().position(|o| o.id == anchor_id);

        match anchor_idx {
            None => Ok(vec![]),
            Some(idx) => {
                let start = idx.saturating_sub(before);
                let end = std::cmp::min(idx + after + 1, obs.len());

                let timeline: Vec<Observation> = obs[start..end]
                    .iter()
                    .filter(|o| {
                        if let Some(ref f) = filter
                            && let Some(ref session) = f.session_id
                            && o.metadata.session_id.as_ref() != Some(session)
                        {
                            return false;
                        }
                        true
                    })
                    .cloned()
                    .collect();

                Ok(timeline)
            }
        }
    }

    async fn search_fts(&self, _query: &str, limit: usize) -> Result<Vec<String>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs.iter().take(limit).map(|o| o.id.clone()).collect())
    }

    async fn search_fts_ranked(&self, _query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs
            .iter()
            .take(limit)
            .map(|o| FtsSearchResult {
                id: o.id.clone(),
                rank: 1.0,
            })
            .collect())
    }

    async fn delete_observation(&self, id: &str) -> Result<()> {
        let mut obs = self.observations.lock().expect("Lock poisoned");
        obs.retain(|o| o.id != id);
        Ok(())
    }

    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs
            .iter()
            .filter(|o| ids.contains(&o.id))
            .cloned()
            .collect())
    }
}

// ============================================================================
// Mock Memory Service
// ============================================================================

use mcb_application::domain_services::memory::MemoryServiceInterface;

pub struct MockMemoryService {
    should_fail: Arc<AtomicBool>,
    error_message: Arc<Mutex<String>>,
    observations: Arc<Mutex<Vec<Observation>>>,
    summaries: Arc<Mutex<Vec<SessionSummary>>>,
}

impl MockMemoryService {
    pub fn new() -> Self {
        Self {
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated memory failure".to_string())),
            observations: Arc::new(Mutex::new(Vec::new())),
            summaries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMemoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryServiceInterface for MockMemoryService {
    async fn store_observation(
        &self,
        content: String,
        observation_type: ObservationType,
        tags: Vec<String>,
        metadata: mcb_domain::entities::memory::ObservationMetadata,
    ) -> Result<(String, bool)> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let obs = Observation {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "mock-project".to_string(),
            content,
            content_hash: "mock-hash".to_string(),
            tags,
            observation_type,
            metadata,
            created_at: chrono::Utc::now().timestamp(),
            embedding_id: None,
        };
        let id = obs.id.clone();
        self.observations.lock().expect("Lock poisoned").push(obs);
        Ok((id, false))
    }

    async fn search_memories(
        &self,
        _query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<MemorySearchResult> = observations
            .iter()
            .filter(|obs| {
                if let Some(ref f) = filter
                    && let Some(ref session) = f.session_id
                    && obs.metadata.session_id.as_ref() != Some(session)
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|obs| MemorySearchResult {
                id: obs.id.clone(),
                observation: obs.clone(),
                similarity_score: 0.95,
            })
            .collect();
        Ok(results)
    }

    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let summaries = self.summaries.lock().expect("Lock poisoned");
        Ok(summaries
            .iter()
            .find(|s| s.session_id == session_id)
            .cloned())
    }

    async fn get_observation(&self, id: &str) -> Result<Option<Observation>> {
        let observations = self.observations.lock().expect("Lock poisoned");
        Ok(observations.iter().find(|o| o.id == id).cloned())
    }

    async fn embed_content(&self, _content: &str) -> Result<Embedding> {
        Ok(Embedding {
            vector: vec![0.0; 384],
            model: "mock-model".to_string(),
            dimensions: 384,
        })
    }

    async fn memory_search(
        &self,
        _query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<MemorySearchIndex> = observations
            .iter()
            .filter(|obs| {
                if let Some(ref f) = filter
                    && let Some(ref session) = f.session_id
                    && obs.metadata.session_id.as_ref() != Some(session)
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|obs| MemorySearchIndex {
                id: obs.id.clone(),
                observation_type: format!("{:?}", obs.observation_type),
                relevance_score: 0.9,
                tags: obs.tags.clone(),
                content_preview: obs
                    .content
                    .chars()
                    .take(120)
                    .chain(std::iter::once('…'))
                    .collect::<String>(),
                session_id: obs.metadata.session_id.clone(),
                repo_id: None,
                file_path: None,
                created_at: obs.created_at,
            })
            .collect();
        Ok(results)
    }

    async fn create_session_summary(
        &self,
        session_id: String,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let summary = SessionSummary {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "mock-project".to_string(),
            session_id,
            topics,
            decisions,
            next_steps,
            key_files,
            created_at: chrono::Utc::now().timestamp(),
        };
        let id = summary.id.clone();
        self.summaries.lock().expect("Lock poisoned").push(summary);
        Ok(id)
    }

    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let anchor_idx = observations.iter().position(|o| o.id == anchor_id);

        match anchor_idx {
            None => Ok(vec![]),
            Some(idx) => {
                let start = idx.saturating_sub(before);
                let end = std::cmp::min(idx + after + 1, observations.len());

                let timeline: Vec<Observation> = observations[start..end]
                    .iter()
                    .filter(|obs| {
                        if let Some(ref f) = filter
                            && let Some(ref session) = f.session_id
                            && obs.metadata.session_id.as_ref() != Some(session)
                        {
                            return false;
                        }
                        true
                    })
                    .cloned()
                    .collect();

                Ok(timeline)
            }
        }
    }

    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<Observation> = observations
            .iter()
            .filter(|obs| ids.contains(&obs.id))
            .cloned()
            .collect();

        Ok(results)
    }
}

// ============================================================================
// Mock VCS Provider
// ============================================================================

/// Mock VCS provider for testing
pub struct MockVcsProvider {
    should_fail: AtomicBool,
}

impl MockVcsProvider {
    pub fn new() -> Self {
        Self {
            should_fail: AtomicBool::new(false),
        }
    }

    pub fn with_failure(self) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        self
    }
}

impl Default for MockVcsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VcsProvider for MockVcsProvider {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(mcb_domain::error::Error::vcs("Mock failure"));
        }
        Ok(VcsRepository {
            id: RepositoryId::new("mock-repo-id".to_string()),
            path: path.to_path_buf(),
            default_branch: "main".to_string(),
            branches: vec!["main".to_string()],
            remote_url: None,
        })
    }

    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId {
        repo.id.clone()
    }

    async fn list_branches(&self, _repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
        Ok(vec![VcsBranch {
            id: uuid::Uuid::new_v4().to_string(),
            name: "main".to_string(),
            head_commit: "abc123".to_string(),
            is_default: true,
            upstream: None,
        }])
    }

    async fn commit_history(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        Ok(vec![VcsCommit {
            id: uuid::Uuid::new_v4().to_string(),
            hash: "abc123".to_string(),
            message: "Initial commit".to_string(),
            author: "Test".to_string(),
            author_email: "test@test.com".to_string(),
            timestamp: 0,
            parent_hashes: vec![],
        }])
    }

    async fn list_files(&self, _repo: &VcsRepository, _branch: &str) -> Result<Vec<PathBuf>> {
        Ok(vec![PathBuf::from("README.md")])
    }

    async fn read_file(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _path: &Path,
    ) -> Result<String> {
        Ok("# Mock File".to_string())
    }

    fn vcs_name(&self) -> &str {
        "mock"
    }

    async fn diff_refs(
        &self,
        _repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff> {
        Ok(RefDiff {
            id: "mock-ref-diff-id".to_string(),
            base_ref: base_ref.to_string(),
            head_ref: head_ref.to_string(),
            files: vec![FileDiff {
                id: "mock-file-diff-id".to_string(),
                path: PathBuf::from("README.md"),
                status: DiffStatus::Modified,
                additions: 5,
                deletions: 2,
            }],
            total_additions: 5,
            total_deletions: 2,
        })
    }
}
