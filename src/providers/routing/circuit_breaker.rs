//! Circuit Breaker Module
//!
//! This module provides circuit breaker functionality using established patterns
//! and libraries, following SOLID principles with proper separation of concerns.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, instrument, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open { opened_at: Instant },
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerState::Closed => write!(f, "closed"),
            CircuitBreakerState::Open { .. } => write!(f, "open"),
            CircuitBreakerState::HalfOpen => write!(f, "half-open"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting recovery
    pub recovery_timeout: Duration,
    /// Number of successes needed to close circuit when half-open
    pub success_threshold: u32,
    /// Maximum requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 10,
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub consecutive_failures: u32,
    pub circuit_opened_count: u32,
    pub circuit_closed_count: u32,
    pub last_failure: Option<Instant>,
    pub last_success: Option<Instant>,
}

/// Persisted circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerSnapshot {
    /// Current state
    pub state: String, // "closed", "open", "half-open"
    /// When the circuit was opened (seconds since saved_at)
    pub opened_at_offset: Option<u64>,
    /// Metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub consecutive_failures: u32,
    pub circuit_opened_count: u32,
    pub circuit_closed_count: u32,
    /// Last saved timestamp
    pub saved_at: u64,
}

/// Trait for circuit breaker
#[async_trait::async_trait]
pub trait CircuitBreakerTrait: Send + Sync {
    async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send;

    async fn state(&self) -> CircuitBreakerState;
    async fn metrics(&self) -> CircuitBreakerMetrics;
}

/// Circuit breaker implementation using established patterns
pub struct CircuitBreaker {
    /// Circuit breaker identifier
    id: String,
    /// Persistence directory
    persistence_dir: PathBuf,
    /// Channel sender for background persistence
    persistence_tx: Option<mpsc::UnboundedSender<()>>,
    /// Shared state
    state: Arc<RwLock<CircuitBreakerInnerState>>,
    /// Configuration
    config: CircuitBreakerConfig,
}

#[derive(Debug)]
struct CircuitBreakerInnerState {
    current_state: CircuitBreakerState,
    metrics: CircuitBreakerMetrics,
    half_open_request_count: u32,
}

impl Clone for CircuitBreakerInnerState {
    fn clone(&self) -> Self {
        Self {
            current_state: self.current_state,
            metrics: self.metrics.clone(),
            half_open_request_count: self.half_open_request_count,
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub async fn new(id: impl Into<String>) -> Self {
        Self::with_config(id, CircuitBreakerConfig::default()).await
    }

    /// Create a new circuit breaker with custom configuration
    pub async fn with_config(id: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let id = id.into();
        let persistence_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".context")
            .join("circuit_breakers");

        let state = Arc::new(RwLock::new(CircuitBreakerInnerState {
            current_state: CircuitBreakerState::Closed,
            metrics: CircuitBreakerMetrics::default(),
            half_open_request_count: 0,
        }));

        let mut cb = Self {
            id,
            persistence_dir,
            persistence_tx: None,
            state,
            config,
        };

        // Try to load persisted state
        if let Ok(Some(snapshot)) = cb.load_snapshot().await {
            cb.apply_snapshot(snapshot).await;
        }

        // Start background persistence task
        cb.start_persistence_task();

        cb
    }

    fn start_persistence_task(&mut self) {
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        self.persistence_tx = Some(tx);

        let id = self.id.clone();
        let persistence_dir = self.persistence_dir.clone();
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            // Debounce persistence calls to every 5 seconds
            let mut last_save = Instant::now();
            let debounce_duration = Duration::from_secs(5);

            while rx.recv().await.is_some() {
                if last_save.elapsed() >= debounce_duration {
                    let snapshot_res = {
                        let state_guard = state.read().await;
                        Self::create_snapshot_static(&id, &state_guard)
                    };

                    if let Ok(snapshot) = snapshot_res {
                        let _ = Self::save_snapshot_static(&id, &persistence_dir, &snapshot).await;
                    }
                    last_save = Instant::now();
                }
            }
        });
    }

    async fn apply_snapshot(&self, snapshot: CircuitBreakerSnapshot) {
        let mut state = self.state.write().await;
        state.current_state = match snapshot.state.as_str() {
            "closed" => CircuitBreakerState::Closed,
            "open" => {
                let opened_at = snapshot
                    .opened_at_offset
                    .map(|offset| {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let saved_at = snapshot.saved_at;
                        let elapsed_since_saved = now.saturating_sub(saved_at);
                        Instant::now()
                            .checked_sub(Duration::from_secs(offset + elapsed_since_saved))
                            .unwrap_or_else(Instant::now)
                    })
                    .unwrap_or_else(Instant::now);
                CircuitBreakerState::Open { opened_at }
            }
            "half-open" => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Closed,
        };

        state.metrics = CircuitBreakerMetrics {
            total_requests: snapshot.total_requests,
            successful_requests: snapshot.successful_requests,
            failed_requests: snapshot.failed_requests,
            rejected_requests: snapshot.rejected_requests,
            consecutive_failures: snapshot.consecutive_failures,
            circuit_opened_count: snapshot.circuit_opened_count,
            circuit_closed_count: snapshot.circuit_closed_count,
            last_failure: None,
            last_success: None,
        };
    }

    fn create_snapshot_static(
        _id: &str,
        state: &CircuitBreakerInnerState,
    ) -> Result<CircuitBreakerSnapshot> {
        let (state_str, opened_at_offset) = match state.current_state {
            CircuitBreakerState::Closed => ("closed", None),
            CircuitBreakerState::Open { opened_at } => {
                ("open", Some(opened_at.elapsed().as_secs()))
            }
            CircuitBreakerState::HalfOpen => ("half-open", None),
        };

        Ok(CircuitBreakerSnapshot {
            state: state_str.to_string(),
            opened_at_offset,
            total_requests: state.metrics.total_requests,
            successful_requests: state.metrics.successful_requests,
            failed_requests: state.metrics.failed_requests,
            rejected_requests: state.metrics.rejected_requests,
            consecutive_failures: state.metrics.consecutive_failures,
            circuit_opened_count: state.metrics.circuit_opened_count,
            circuit_closed_count: state.metrics.circuit_closed_count,
            saved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    async fn save_snapshot_static(
        id: &str,
        dir: &PathBuf,
        snapshot: &CircuitBreakerSnapshot,
    ) -> Result<()> {
        if !dir.exists() {
            tokio::fs::create_dir_all(dir).await.map_err(|e| Error::io(e.to_string()))?;
        }

        let file_path = dir.join(format!("{}.json", id));
        let content =
            serde_json::to_string(snapshot).map_err(|e| Error::internal(e.to_string()))?;
        tokio::fs::write(file_path, content).await.map_err(|e| Error::io(e.to_string()))?;
        Ok(())
    }

    async fn load_snapshot(&self) -> Result<Option<CircuitBreakerSnapshot>> {
        let file_path = self.persistence_dir.join(format!("{}.json", self.id));
        if !file_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| Error::io(e.to_string()))?;
        let snapshot =
            serde_json::from_str(&content).map_err(|e| Error::internal(e.to_string()))?;
        Ok(Some(snapshot))
    }

    fn request_save(&self) {
        if let Some(tx) = &self.persistence_tx {
            let _ = tx.send(());
        }
    }

    async fn check_state_transition(&self) {
        let mut state = self.state.write().await;

        if let CircuitBreakerState::Open { opened_at } = state.current_state
            && opened_at.elapsed() >= self.config.recovery_timeout
        {
            info!("Circuit breaker {} transitioning to Half-Open", self.id);
            state.current_state = CircuitBreakerState::HalfOpen;
            state.half_open_request_count = 0;
            self.request_save();
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        state.metrics.total_requests += 1;
        state.metrics.successful_requests += 1;
        state.metrics.consecutive_failures = 0;
        state.metrics.last_success = Some(Instant::now());

        if state.current_state == CircuitBreakerState::HalfOpen {
            state.half_open_request_count += 1;
            if state.half_open_request_count >= self.config.success_threshold {
                info!("Circuit breaker {} transitioning to Closed", self.id);
                state.current_state = CircuitBreakerState::Closed;
                state.metrics.circuit_closed_count += 1;
                self.request_save();
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        state.metrics.total_requests += 1;
        state.metrics.failed_requests += 1;
        state.metrics.consecutive_failures += 1;
        state.metrics.last_failure = Some(Instant::now());

        if state.current_state == CircuitBreakerState::Closed {
            if state.metrics.consecutive_failures >= self.config.failure_threshold {
                warn!("Circuit breaker {} transitioning to Open", self.id);
                state.current_state = CircuitBreakerState::Open {
                    opened_at: Instant::now(),
                };
                state.metrics.circuit_opened_count += 1;
                self.request_save();
            }
        } else if state.current_state == CircuitBreakerState::HalfOpen {
            warn!(
                "Circuit breaker {} failing in Half-Open, transitioning back to Open",
                self.id
            );
            state.current_state = CircuitBreakerState::Open {
                opened_at: Instant::now(),
            };
            state.metrics.circuit_opened_count += 1;
            self.request_save();
        }
    }
}

#[async_trait::async_trait]
impl CircuitBreakerTrait for CircuitBreaker {
    #[instrument(skip(self, f), fields(id = %self.id))]
    async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        self.check_state_transition().await;

        let current_state = {
            let state = self.state.read().await;
            state.current_state
        };

        match current_state {
            CircuitBreakerState::Open { .. } => {
                let mut state = self.state.write().await;
                state.metrics.rejected_requests += 1;
                Err(Error::generic(format!(
                    "Circuit breaker {} is open",
                    self.id
                )))
            }
            CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen => {
                // In half-open, we limit the number of concurrent requests
                if current_state == CircuitBreakerState::HalfOpen {
                    let mut state = self.state.write().await;
                    if state.half_open_request_count >= self.config.half_open_max_requests {
                        state.metrics.rejected_requests += 1;
                        return Err(Error::generic(format!(
                            "Circuit breaker {} is half-open and reached max requests",
                            self.id
                        )));
                    }
                    state.half_open_request_count += 1;
                }

                // Call the function
                match f().await {
                    Ok(result) => {
                        self.on_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        // We only count certain types of errors as failures for the circuit breaker
                        // For example, network errors or timeouts should count, but business logic
                        // errors or "not found" should probably not.
                        // For now, count all errors as failures.
                        self.on_failure().await;
                        Err(e)
                    }
                }
            }
        }
    }

    async fn state(&self) -> CircuitBreakerState {
        self.state.read().await.current_state
    }

    async fn metrics(&self) -> CircuitBreakerMetrics {
        self.state.read().await.metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new("test");
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_successful_operations() {
        let cb = CircuitBreaker::new("test_success");
        let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
        assert_eq!(cb.metrics().await.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config("test_failure", config);

        // First failure
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);

        // Second failure - should open
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
        assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(10),
            success_threshold: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config("test_reset", config);

        // Open circuit
        let _: Result<()> = cb.call(|| async { Err(Error::generic("fail")) }).await;
        assert!(matches!(cb.state().await, CircuitBreakerState::Open { .. }));

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Should transition to half-open and then close on success
        let result: Result<i32> = cb.call(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }
}
