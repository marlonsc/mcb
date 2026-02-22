//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin request handlers shared code
//!
//! Contains shared state and type definitions used across admin modules.

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::ports::CacheProvider;
use mcb_domain::ports::EventBusProvider;
use mcb_domain::ports::{
    IndexingOperationsInterface, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_domain::ports::{
    IssueEntityRepository, OrgEntityRepository, PlanEntityRepository, ProjectRepository,
    VcsEntityRepository,
};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::config::watcher::ConfigWatcher;
use mcb_infrastructure::infrastructure::ServiceManager;

use crate::tools::ToolHandlers;

/// Admin handler state containing shared service references
#[derive(Clone)]
pub struct AdminState {
    /// Performance metrics tracker
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations tracker
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Configuration watcher for hot-reload support
    pub config_watcher: Option<Arc<ConfigWatcher>>,
    /// Current configuration snapshot (read-only fallback if watcher unavailable)
    pub current_config: AppConfig,
    /// Configuration file path (for updates)
    pub config_path: Option<PathBuf>,
    /// Shutdown coordinator for graceful shutdown
    pub shutdown_coordinator: Option<Arc<dyn ShutdownCoordinator>>,
    /// Default shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
    /// Event bus for SSE streaming
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Service manager for lifecycle control
    pub service_manager: Option<Arc<ServiceManager>>,
    /// Cache provider for stats
    pub cache: Option<Arc<dyn CacheProvider>>,
    /// Project workflow repository used by admin CRUD pages.
    pub project_workflow: Option<Arc<dyn ProjectRepository>>,
    /// VCS entity repository used by admin CRUD pages.
    pub vcs_entity: Option<Arc<dyn VcsEntityRepository>>,
    /// Plan entity repository used by admin CRUD pages.
    pub plan_entity: Option<Arc<dyn PlanEntityRepository>>,
    /// Issue entity repository used by admin CRUD pages.
    pub issue_entity: Option<Arc<dyn IssueEntityRepository>>,
    /// Organization entity repository used by admin CRUD pages.
    pub org_entity: Option<Arc<dyn OrgEntityRepository>>,
    /// Unified MCP tool handlers used to execute admin operations via command gateway.
    pub tool_handlers: Option<ToolHandlers>,
}
