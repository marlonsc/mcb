//! Provider Connection Tracker for managing active operations during restart
//!
//! This module tracks active connections/operations per provider to enable
//! graceful connection draining during provider restarts.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Connection guard that decrements connection count when dropped
pub struct ConnectionGuard {
    provider_id: String,
    tracker: ProviderConnectionTracker,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.tracker.decrement(&self.provider_id);
    }
}

/// Tracks active connections for providers
#[derive(Clone)]
pub struct ProviderConnectionTracker {
    /// Active connections per provider
    active_connections: Arc<DashMap<String, Arc<AtomicU32>>>,
}

impl ProviderConnectionTracker {
    /// Create a new connection tracker
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(DashMap::new()),
        }
    }

    /// Start tracking a connection for a provider
    pub fn track_connection(&self, provider_id: &str) -> ConnectionGuard {
        let key = provider_id.to_string();

        // Get or create atomic counter for this provider
        let counter = self
            .active_connections
            .entry(key.clone())
            .or_insert_with(|| Arc::new(AtomicU32::new(0)))
            .clone();

        // Increment connection count
        counter.fetch_add(1, Ordering::SeqCst);
        debug!("[TRACKER] Connection started for {}", provider_id);

        ConnectionGuard {
            provider_id: key,
            tracker: self.clone(),
        }
    }

    /// Decrement active connection count (called by ConnectionGuard drop)
    fn decrement(&self, provider_id: &str) {
        if let Some(entry) = self.active_connections.get(provider_id) {
            let count = entry.fetch_sub(1, Ordering::SeqCst);
            debug!(
                "[TRACKER] Connection ended for {} (remaining: {})",
                provider_id,
                count - 1
            );
        }
    }

    /// Get current active connection count
    pub fn active_count(&self, provider_id: &str) -> u32 {
        self.active_connections
            .get(provider_id)
            .map(|counter| counter.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Wait for all connections to drain with optional timeout
    pub async fn wait_for_drain(&self, provider_id: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(100);

        loop {
            let remaining = self.active_count(provider_id);
            if remaining == 0 {
                debug!("[TRACKER] All connections drained for {}", provider_id);
                return true;
            }

            if start.elapsed() > timeout {
                warn!(
                    "[TRACKER] Timeout waiting for {} connections to drain for {}",
                    remaining, provider_id
                );
                return false;
            }

            sleep(check_interval).await;
        }
    }

    /// Force close all connections for a provider (used as fallback)
    pub fn close_all(&self, provider_id: &str) {
        if let Some(entry) = self.active_connections.get_mut(provider_id) {
            entry.store(0, Ordering::SeqCst);
        }
        debug!(
            "[TRACKER] Forced close of all connections for {}",
            provider_id
        );
    }
}

impl Default for ProviderConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_tracking() {
        let tracker = ProviderConnectionTracker::new();

        // Initially no connections
        assert_eq!(tracker.active_count("provider1"), 0);

        // Track a connection
        let _guard = tracker.track_connection("provider1");
        assert_eq!(tracker.active_count("provider1"), 1);

        // Drop guard to decrement
        drop(_guard);
        assert_eq!(tracker.active_count("provider1"), 0);
    }

    #[tokio::test]
    async fn test_wait_for_drain_immediate() {
        let tracker = ProviderConnectionTracker::new();

        // No connections, should drain immediately
        let drained = tracker
            .wait_for_drain("provider1", Duration::from_secs(1))
            .await;
        assert!(drained);
    }

    #[tokio::test]
    async fn test_wait_for_drain_timeout() {
        let tracker = ProviderConnectionTracker::new();

        // Create a connection that won't drain
        let _guard = tracker.track_connection("provider1");

        // Should timeout
        let drained = tracker
            .wait_for_drain("provider1", Duration::from_millis(200))
            .await;
        assert!(!drained);

        // Clean up
        drop(_guard);
    }

    #[tokio::test]
    async fn test_wait_for_drain_with_completion() {
        let tracker = ProviderConnectionTracker::new();
        let tracker_clone = tracker.clone();

        let tracker_for_task = tracker.clone();
        tokio::spawn(async move {
            // Hold connection for 100ms
            let _guard = tracker_for_task.track_connection("provider1");
            tokio::time::sleep(Duration::from_millis(100)).await;
        });

        // Wait for drain with sufficient timeout
        let drained = tracker_clone
            .wait_for_drain("provider1", Duration::from_secs(1))
            .await;
        assert!(drained);
    }
}
