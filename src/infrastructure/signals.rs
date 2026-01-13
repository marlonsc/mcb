//! Unix Signal Handlers for Graceful Server Management
//!
//! This module provides signal handling for:
//! - SIGHUP: Reload configuration
//! - SIGTERM: Graceful shutdown
//! - SIGUSR1: Trigger binary respawn
//! - SIGINT (Ctrl+C): Graceful shutdown
//!
//! Uses `CancellationToken` for async-native shutdown signaling.

use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use anyhow::Result;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

/// Signal handler configuration
#[derive(Debug, Clone)]
pub struct SignalConfig {
    /// Enable SIGHUP handling for config reload
    pub handle_sighup: bool,
    /// Enable SIGUSR1 handling for binary respawn
    pub handle_sigusr1: bool,
    /// Enable SIGTERM handling for graceful shutdown
    pub handle_sigterm: bool,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            handle_sighup: true,
            handle_sigusr1: true,
            handle_sigterm: true,
        }
    }
}

/// Signal handler that publishes events to the event bus
///
/// Uses `CancellationToken` for async-native shutdown signaling.
pub struct SignalHandler {
    event_bus: SharedEventBusProvider,
    config: SignalConfig,
    cancel_token: CancellationToken,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new(event_bus: SharedEventBusProvider, config: SignalConfig) -> Self {
        Self {
            event_bus,
            config,
            cancel_token: CancellationToken::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(event_bus: SharedEventBusProvider) -> Self {
        Self::new(event_bus, SignalConfig::default())
    }

    /// Check if the handler is running
    pub fn is_running(&self) -> bool {
        !self.cancel_token.is_cancelled()
    }

    /// Start listening for signals
    ///
    /// This function spawns an async task that listens for Unix signals
    /// and publishes appropriate events to the event bus.
    pub async fn start(&self) -> Result<()> {
        if self.cancel_token.is_cancelled() {
            warn!("Signal handler was stopped and cannot be restarted");
            return Ok(());
        }

        let event_bus = Arc::clone(&self.event_bus);
        let config = self.config.clone();
        let cancel_token = self.cancel_token.clone();

        tokio::spawn(async move {
            if let Err(e) = run_signal_loop(event_bus, config, cancel_token).await {
                warn!("Signal handler error: {}", e);
            }
        });

        info!("Signal handlers registered (SIGHUP, SIGTERM, SIGUSR1)");
        Ok(())
    }

    /// Stop the signal handler
    pub fn stop(&self) {
        self.cancel_token.cancel();
        info!("Signal handler stopped");
    }
}

/// Internal signal loop
async fn run_signal_loop(
    event_bus: SharedEventBusProvider,
    config: SignalConfig,
    cancel_token: CancellationToken,
) -> Result<()> {
    // Create signal streams
    let mut sighup = if config.handle_sighup {
        Some(signal(SignalKind::hangup())?)
    } else {
        None
    };

    let mut sigterm = if config.handle_sigterm {
        Some(signal(SignalKind::terminate())?)
    } else {
        None
    };

    let mut sigusr1 = if config.handle_sigusr1 {
        Some(signal(SignalKind::user_defined1())?)
    } else {
        None
    };

    loop {
        tokio::select! {
            biased;

            // Check for cancellation first
            _ = cancel_token.cancelled() => {
                debug!("Signal handler received cancellation signal");
                break;
            }

            // Handle SIGHUP - Reload configuration
            Some(_) = async {
                match &mut sighup {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGHUP, triggering configuration reload");
                if let Err(e) = event_bus.publish(SystemEvent::Reload).await {
                    warn!("Failed to publish Reload event: {}", e);
                }
            }

            // Handle SIGTERM - Graceful shutdown
            Some(_) = async {
                match &mut sigterm {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGTERM, initiating graceful shutdown");
                if let Err(e) = event_bus.publish(SystemEvent::Shutdown).await {
                    warn!("Failed to publish Shutdown event: {}", e);
                }
                break;
            }

            // Handle SIGUSR1 - Binary respawn
            Some(_) = async {
                match &mut sigusr1 {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGUSR1, triggering binary respawn");
                if let Err(e) = event_bus.publish(SystemEvent::Respawn).await {
                    warn!("Failed to publish Respawn event: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Convenience function to start signal handlers with event bus
pub async fn start_signal_handlers(event_bus: SharedEventBusProvider) -> Result<SignalHandler> {
    let handler = SignalHandler::with_defaults(event_bus);
    handler.start().await?;
    Ok(handler)
}
