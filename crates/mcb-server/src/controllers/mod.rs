//! HTTP controllers for the MCB admin panel and API endpoints.

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
/// Web UI pages (dashboard, config, health, jobs, browse, 404).
pub mod web;
