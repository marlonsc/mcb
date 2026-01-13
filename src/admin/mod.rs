//! Admin interface module for MCP Context Browser
//!
//! Provides comprehensive administration capabilities including:
//! - API endpoints and routing
//! - Authentication and authorization
//! - Configuration management
//! - Web interface and templates
//! - HTTP handlers for all admin operations
//! - Business logic services

pub mod api;
pub mod auth;
pub mod config;
pub mod config_keys;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod service;
pub mod web;

// Re-export commonly used types
pub use api::AdminApi;
pub use auth::{AuthConfig, User};
pub use config::AdminConfig;
pub use models::{AdminState, ApiResponse, SystemConfig};
pub use service::AdminService;
