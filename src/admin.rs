//! Enterprise Administration & Monitoring Platform
//!
//! Provides a comprehensive web-based administration, configuration, and monitoring
//! interface for MCP Context Browser with advanced features for enterprise deployments.

pub mod api;
pub mod auth;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod service;
pub mod web;
pub mod config;

pub use routes::create_admin_router;
pub use config::{AdminApi, AdminConfig};
