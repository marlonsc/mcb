//! Admin Interface
//!
//! Administrative interfaces for system monitoring and management.
//! Uses domain ports to maintain Clean Architecture separation.
//!
//! ## Architecture
//!
//! The admin module follows the same Clean Architecture pattern as the rest
//! of the server:
//!
//! - **Domain Ports** (`mcb_domain::ports::admin`): Define the interfaces
//! - **Infrastructure Adapters** (`mcb_infrastructure::adapters::admin`): Implementations
//! - **Server Handlers** (this module): HTTP handlers and routes
//!
//! ## Endpoints
//!
//! | Path | Method | Description |
//! |------|--------|-------------|
//! | `/health` | GET | Health check with uptime |
//! | `/metrics` | GET | Performance metrics |
//! | `/indexing` | GET | Indexing operations status |
//! | `/ready` | GET | Kubernetes readiness probe |
//! | `/live` | GET | Kubernetes liveness probe |

pub mod api;
pub mod auth;
pub mod config;
pub mod handlers;
pub mod models;
pub mod routes;

// Re-export main types
pub use api::{AdminApi, AdminApiConfig};
pub use config::{
    ConfigReloadResponse, ConfigResponse, ConfigSectionUpdateRequest, ConfigSectionUpdateResponse,
    SanitizedConfig,
};
pub use handlers::AdminState;
pub use models::{AdminActionResponse, CollectionStats, ServerInfo};
pub use routes::{admin_router, admin_router_with_prefix};
