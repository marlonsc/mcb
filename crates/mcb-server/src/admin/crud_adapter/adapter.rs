//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Entity CRUD adapter trait definition.

use std::collections::HashSet;

use async_trait::async_trait;
use serde_json::Value;

use crate::admin::web::filter::{FilterParams, FilteredResult};
use crate::admin::web::pipeline::apply_filter_pipeline;

/// Async CRUD operations that map entity slugs to domain service calls.
#[async_trait]
pub trait EntityCrudAdapter: Send + Sync {
    /// List all records for this entity.
    async fn list_all(&self) -> Result<Vec<Value>, String>;
    /// Get a single record by its primary key.
    async fn get_by_id(&self, id: &str) -> Result<Value, String>;
    /// Create a record from raw JSON form data.
    async fn create_from_json(&self, data: Value) -> Result<Value, String>;
    /// Update a record from raw JSON form data.
    async fn update_from_json(&self, data: Value) -> Result<(), String>;
    /// Delete a record by its primary key.
    async fn delete_by_id(&self, id: &str) -> Result<(), String>;

    /// List records with in-memory filtering, sorting, and pagination.
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        match self.list_all().await {
            Ok(records) => Ok(apply_filter_pipeline(records, params, valid_sort_fields)),
            Err(e) => Err(e),
        }
    }
}
