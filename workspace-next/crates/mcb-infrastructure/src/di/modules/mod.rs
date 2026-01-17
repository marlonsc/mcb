//! DI Module Organization - Hierarchical by Domain (Shaku Strict Pattern)
//!
//! This module implements a strict Shaku-based hierarchical module system
//! following Clean Architecture and Domain-Driven Design principles.
//!
//! ## Shaku Module Hierarchy Pattern
//!
//! ```text
//! McpModule (Root - composes all modules)
//! ├── InfrastructureModule (core services - no dependencies)
//! ├── ServerModule (MCP server components - no dependencies)
//! ├── AdaptersModule (external integrations - no dependencies)
//! ├── ApplicationModule (business logic - placeholder)
//! └── AdminModule (admin services - placeholder)
//! ```
//!
//! ## Note on Current Implementation
//!
//! Many services are created via factory patterns at runtime (see domain_services.rs)
//! rather than through Shaku DI, because they require runtime configuration.
//! The Shaku modules here provide the foundation, with null providers as defaults.
//!
//! ## Module Construction Pattern
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use mcb_infrastructure::di::modules::*;
//!
//! // Build leaf modules
//! let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
//! let server = Arc::new(ServerModuleImpl::builder().build());
//! let adapters = Arc::new(AdaptersModuleImpl::builder().build());
//! let application = Arc::new(ApplicationModuleImpl::builder().build());
//! let admin = Arc::new(AdminModuleImpl::builder().build());
//!
//! // Build root module
//! let root = McpModule::builder(infrastructure, server, adapters, application, admin).build();
//! ```

/// Domain module traits (interfaces)
pub mod traits;

/// Context modules (Clean Architecture pattern)
pub mod cache_module;
pub mod embedding_module;
pub mod data_module;
pub mod language_module;
pub mod usecase_module;

/// Legacy modules (compatibility)
/// Infrastructure module implementation (core infrastructure)
pub mod infrastructure;
/// Server module implementation (MCP server components)
pub mod server;
/// Adapters module implementation (external integrations)
pub mod adapters;
/// Application module implementation (business logic)
pub mod application;
/// Admin module implementation (admin services)
pub mod admin;

/// Domain services factory (runtime service creation)
pub mod domain_services;

pub use adapters::AdaptersModuleImpl;
pub use admin::AdminModuleImpl;
pub use application::ApplicationModuleImpl;
pub use infrastructure::InfrastructureModuleImpl;
pub use server::ServerModuleImpl;
pub use traits::{
    AdaptersModule, AdminModule, ApplicationModule, InfrastructureModule, ServerModule,
};

// Re-export Shaku for convenience
pub use shaku::module;

// Re-export domain services
pub use domain_services::{DomainServicesContainer, DomainServicesFactory};

// ============================================================================
// Root Module Definition (Shaku Strict Pattern)
// ============================================================================


// Import provider traits from mcb-domain
use mcb_application::ports::providers::{EmbeddingProvider, VectorStoreProvider};

// TODO: Implement AppModule for clean composition
// Currently using McpModule for compatibility

// ============================================================================
// Root Module Definition
// ============================================================================

