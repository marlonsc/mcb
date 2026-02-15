//! Admin module
//!
//! Provides administrative endpoints and functionality for server management.

pub mod api;
pub mod auth;
/// Admin browsing endpoints and query responses.
pub mod browse;
pub mod browse_handlers;
/// Cache administration endpoints.
pub mod cache;
pub mod config;
pub mod config_handlers;
/// Service control endpoints such as shutdown.
pub mod control;
pub mod crud_adapter;
pub mod handlers;
/// Health and metrics endpoints.
pub mod health;
/// Job status endpoints.
pub mod jobs;
pub mod lifecycle_handlers;
pub mod models;
pub mod propagation;
pub mod registry;
pub use api::{AdminApi, AdminApiConfig};
pub use registry::AdminRegistry;

pub mod routes;
pub mod sse;
pub mod web;
