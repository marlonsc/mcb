//! Unified CRUD adapter implementation.

use std::collections::HashSet;

use async_trait::async_trait;
use rmcp::model::CallToolRequestParams;
use serde_json::Value;

use crate::admin::web::filter::{FilterParams, FilteredResult};
use crate::admin::web::pipeline::apply_filter_pipeline;
use crate::args::{EntityAction, EntityArgs, EntityResource};
use crate::tools::{ToolExecutionContext, ToolHandlers, route_tool_call};

use super::adapter::EntityCrudAdapter;
use super::mapper::{
    apply_parent_scope, base_entity_args, build_entity_arguments, extract_project_id,
};
use crate::utils::text::extract_text;

/// Routes CRUD operations for any entity type through the unified tool dispatch.
#[derive(Clone)]
pub struct UnifiedEntityCrudAdapter {
    /// Which entity resource this adapter manages.
    pub resource: EntityResource,
    /// FK field name when the resource is scoped under a parent (e.g. `"team_id"`).
    pub parent_field: Option<&'static str>,
    /// Shared tool handlers used to dispatch entity operations.
    pub handlers: ToolHandlers,
}

impl UnifiedEntityCrudAdapter {
    async fn execute(&self, args: EntityArgs) -> Result<Value, String> {
        let arguments = build_entity_arguments(args);

        let request = CallToolRequestParams {
            name: "entity".into(),
            arguments: Some(arguments),
            task: None,
            meta: None,
        };

        let result = route_tool_call(request, &self.handlers, ToolExecutionContext::default())
            .await
            .map_err(|e| format!("entity dispatch failed: {}", e.message))?;

        let text = extract_text(&result.content);
        if result.is_error.unwrap_or(false) {
            return Err(if text.is_empty() {
                "entity operation failed".to_owned()
            } else {
                text
            });
        }

        if text.trim().is_empty() {
            Ok(Value::Null)
        } else {
            match serde_json::from_str(&text) {
                Ok(json) => Ok(json),
                Err(_) => Ok(Value::String(text)),
            }
        }
    }

    async fn list_with_parent(&self, parent_id: Option<&str>) -> Result<Vec<Value>, String> {
        let mut args = base_entity_args(self.resource, EntityAction::List);
        apply_parent_scope(&mut args, self.parent_field, parent_id);
        match self.execute(args).await? {
            Value::Array(items) => Ok(items),
            Value::Null => Ok(Vec::new()),
            other @ (Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Object(_)) => {
                Err(format!("expected list response, got: {other}"))
            }
        }
    }
}

#[async_trait]
impl EntityCrudAdapter for UnifiedEntityCrudAdapter {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        if self.parent_field.is_some() {
            return Ok(Vec::new());
        }
        self.list_with_parent(None).await
    }

    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (
            self.parent_field,
            params.parent_field.as_deref(),
            params.parent_id.as_deref(),
        ) {
            (Some(expected), Some(actual), Some(parent_id)) if expected == actual => {
                self.list_with_parent(Some(parent_id)).await?
            }
            (Some(_), _, _) => Vec::new(),
            (None, _, _) => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }

    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        if matches!(
            self.resource,
            EntityResource::TeamMember | EntityResource::LabelAssignment
        ) {
            return Err("resource has a composite key and cannot be fetched by id".to_owned());
        }

        let mut args = base_entity_args(self.resource, EntityAction::Get);
        args.id = Some(id.to_owned());
        self.execute(args).await
    }

    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let mut args = base_entity_args(self.resource, EntityAction::Create);
        if matches!(self.resource, EntityResource::Repository) {
            args.project_id = extract_project_id(&data);
            if args.project_id.is_none() {
                return Err("project_id is required for repository create".to_owned());
            }
        }
        args.data = Some(data);
        self.execute(args).await
    }

    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let mut args = base_entity_args(self.resource, EntityAction::Update);
        if matches!(self.resource, EntityResource::Repository) {
            args.project_id = extract_project_id(&data);
            if args.project_id.is_none() {
                return Err("project_id is required for repository update".to_owned());
            }
        }
        args.data = Some(data);
        let _ = self.execute(args).await?;
        Ok(())
    }

    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        if matches!(
            self.resource,
            EntityResource::TeamMember | EntityResource::LabelAssignment
        ) {
            return Err("resource has a composite key and cannot be deleted by id".to_owned());
        }

        let mut args = base_entity_args(self.resource, EntityAction::Delete);
        if matches!(self.resource, EntityResource::Repository) {
            let existing = self.get_by_id(id).await?;
            let project_id = existing
                .get("project_id")
                .and_then(Value::as_str)
                .ok_or_else(|| "repository record missing project_id".to_owned())?;
            args.project_id = Some(project_id.to_owned());
        }
        args.id = Some(id.to_owned());
        let _ = self.execute(args).await?;
        Ok(())
    }
}
