//! Admin Service Implementations
//!
//! Real and null implementations of admin port traits.
//! Moved from mcb-providers to mcb-infrastructure per Clean Architecture.

use dashmap::DashMap;
use mcb_domain::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
use mcb_domain::value_objects::{CollectionId, OperationId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;
use uuid::Uuid;

// ============================================================================
// Performance Metrics - Real Implementation
// ============================================================================

/// Atomic performance metrics tracker
///
/// Thread-safe implementation of PerformanceMetricsInterface using atomic operations.
/// Tracks queries, response times, cache hits, and active connections.
pub struct AtomicPerformanceMetrics {
    /// Server start time for uptime calculation
    start_time: Instant,

    /// Total number of queries processed
    total_queries: AtomicU64,

    /// Number of successful queries
    successful_queries: AtomicU64,

    /// Number of failed queries
    failed_queries: AtomicU64,

    /// Sum of all response times in milliseconds
    total_response_time_ms: AtomicU64,

    /// Number of cache hits
    cache_hits: AtomicU64,

    /// Current active connection count
    active_connections: AtomicU32,
}

impl AtomicPerformanceMetrics {
    /// Create a new performance metrics tracker
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for AtomicPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetricsInterface for AtomicPerformanceMetrics {
    fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_ms
            .fetch_add(response_time_ms, Ordering::Relaxed);

        if success {
            self.successful_queries.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }

        if cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn update_active_connections(&self, delta: i64) {
        if delta >= 0 {
            self.active_connections
                .fetch_add(delta as u32, Ordering::Relaxed);
        } else {
            let abs_delta = (-delta) as u32;
            self.active_connections.fetch_sub(
                abs_delta.min(self.active_connections.load(Ordering::Relaxed)),
                Ordering::Relaxed,
            );
        }
    }

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        let total = self.total_queries.load(Ordering::Relaxed);
        let successful = self.successful_queries.load(Ordering::Relaxed);
        let failed = self.failed_queries.load(Ordering::Relaxed);
        let total_time = self.total_response_time_ms.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);

        let average_response_time_ms = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        let cache_hit_rate = if total > 0 {
            cache_hits as f64 / total as f64
        } else {
            0.0
        };

        PerformanceMetricsData {
            total_queries: total,
            successful_queries: successful,
            failed_queries: failed,
            average_response_time_ms,
            cache_hit_rate,
            active_connections: self.active_connections.load(Ordering::Relaxed) as usize,
            uptime_seconds: self.uptime_secs(),
        }
    }
}

// ============================================================================
// Indexing Operations - Real Implementation
// ============================================================================

/// Default indexing operations tracker
///
/// Thread-safe implementation using DashMap for concurrent access.
pub struct DefaultIndexingOperations {
    /// Active indexing operations by ID
    operations: Arc<DashMap<OperationId, IndexingOperation>>,
}

impl DefaultIndexingOperations {
    /// Create a new indexing operations tracker
    pub fn new() -> Self {
        Self {
            operations: Arc::new(DashMap::new()),
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Start tracking a new indexing operation (inherent impl; trait delegates here).
    pub fn start_operation_internal(
        &self,
        collection: &CollectionId,
        total_files: usize,
    ) -> OperationId {
        let id = OperationId::new(Uuid::new_v4().to_string());
        let operation = IndexingOperation {
            id: id.clone(),
            collection: collection.clone(),
            current_file: None,
            status: mcb_domain::ports::admin::IndexingStatus::Starting,
            total_files,
            processed_files: 0,
            started_at: chrono::Utc::now(),
        };
        self.operations.insert(id.clone(), operation);
        id
    }

    /// Update progress for an operation
    pub fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    ) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.current_file = current_file;
            op.processed_files = processed;
        }
    }

    /// Complete and remove an operation
    pub fn complete_operation(&self, operation_id: &OperationId) {
        self.operations.remove(operation_id);
    }

    /// Check if any operations are in progress
    pub fn has_active_operations(&self) -> bool {
        !self.operations.is_empty()
    }

    /// Get count of active operations
    pub fn active_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for DefaultIndexingOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexingOperationsInterface for DefaultIndexingOperations {
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation> {
        self.operations
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId {
        self.start_operation_internal(collection, total_files)
    }

    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    ) {
        DefaultIndexingOperations::update_progress(self, operation_id, current_file, processed);
    }

    fn complete_operation(&self, operation_id: &OperationId) {
        DefaultIndexingOperations::complete_operation(self, operation_id);
    }
}
