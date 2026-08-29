//! HTTP controllers for the MCB admin panel and API endpoints.
//!
//! All controller handlers return `loco_rs::Error` as the `Err` variant of
//! `axum::response::Result`. The `loco_rs::Error` type wraps the full
//! `loco_rs::Error` struct (>128 bytes), which trips clippy's
//! `result_large_err` lint. This is an inherent property of the Loco
//! framework's error type and is not actionable at the call site.
#![allow(clippy::result_large_err)]

/// Admin API (config, dashboard).
pub mod admin;
/// Admin config loading (sea-orm-pro).
pub mod admin_config;
/// Collections API (vector store browser).
pub mod collections_api;
/// GraphQL API.
pub mod graphql;
/// Health API (provider health checks).
pub mod health_api;
/// Jobs API (indexing and validation operations).
pub mod jobs_api;
/// Reusable Axum route construction for admin/UI/MCP endpoints.
pub mod routes;
/// Web UI pages (dashboard, config, health, jobs, browse, 404).
pub mod web;
