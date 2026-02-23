//! Lifecycle helpers — shutdown coordination.
//!
//! The `ServiceManager` was removed (Loco manages service lifecycle).
//! Only `DefaultShutdownCoordinator` remains for the test bootstrap.

use std::sync::atomic::{AtomicBool, Ordering};

use mcb_domain::ports::ShutdownCoordinator;
use tokio::sync::Notify;

/// Default implementation of `ShutdownCoordinator` using atomics and Notify.
pub struct DefaultShutdownCoordinator {
    shutdown_signal: AtomicBool,
    notify: Notify,
}

impl DefaultShutdownCoordinator {
    pub fn new() -> Self {
        Self {
            shutdown_signal: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }
}

impl Default for DefaultShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DefaultShutdownCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultShutdownCoordinator")
            .field("is_shutting_down", &self.is_shutting_down())
            .finish()
    }
}

impl ShutdownCoordinator for DefaultShutdownCoordinator {
    fn signal_shutdown(&self) {
        mcb_domain::info!("lifecycle", "Shutdown signal received");
        self.shutdown_signal.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    fn is_shutting_down(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }
}
