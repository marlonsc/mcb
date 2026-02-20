//! Admin module
//!
//! **Documentation**: [`docs/modules/admin.md`](../../../../docs/modules/admin.md)
//!
//! Provides administrative endpoints and functionality for server management.

pub mod api;
mod api_launch;
pub mod auth;
/// Admin browsing endpoints and query responses.
pub mod browse;
pub mod browse_handlers;
mod browse_models;
mod browse_runtime;
/// Cache administration endpoints.
pub mod cache;
/// Configuration management endpoints for viewing and updating server settings.
pub mod config;

/// Service control endpoints such as shutdown.
pub mod control;
pub mod crud_adapter;
pub mod handlers;
/// Health and metrics endpoints.
pub mod health;
/// Job status endpoints.
pub mod jobs;
pub mod lifecycle_handlers;
mod lifecycle_models;
pub mod models;
pub mod propagation;
pub mod registry;
pub use api::{AdminApi, AdminApiConfig};
pub use registry::AdminRegistry;

pub mod routes;
pub mod sse;
pub mod web;
