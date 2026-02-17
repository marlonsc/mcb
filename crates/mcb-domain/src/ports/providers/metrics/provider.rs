#![allow(missing_docs)]

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;

use super::metric_defs::{MetricLabels, MetricsResult, labels_from};

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
