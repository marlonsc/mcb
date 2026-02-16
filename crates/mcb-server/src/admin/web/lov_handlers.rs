//! List-of-Values (LOV) endpoint for populating FK dropdown selects.
//!
//! Returns lightweight `[{id, label}]` JSON arrays suitable for `<select>`
//! elements in the admin UI.  Supports optional `?q=` search filtering and
//! `?parent_id=` scoping for hierarchical relationships.

use rocket::State;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;
use serde_json::Value;

use crate::admin::crud_adapter::resolve_adapter;
use crate::admin::handlers::AdminState;
use crate::admin::registry::{AdminEntityMeta, AdminRegistry};
use crate::constants::limits::DEFAULT_LOV_LIMIT;

/// A single item in a List-of-Values response.
#[derive(Debug, Clone, Serialize)]
pub struct LovItem {
    /// Primary key of the record.
    pub id: String,
    /// Human-readable display label (derived from the entity's display field).
    pub label: String,
}

/// Determines the best display field for an entity by inspecting its schema.
///
/// Returns the name of the first non-id, non-hidden, non-timestamp string
/// field (i.e. `input_type == "text"`).  Falls back to `"id"` when no
/// suitable candidate is found.
fn display_field(entity: &AdminEntityMeta) -> String {
    let fields = entity.fields();
    fields
        .iter()
        .find(|f| is_display_field_candidate(f))
        .map_or_else(|| "id".to_owned(), |f| f.name.clone())
}

fn is_display_field_candidate(field: &crate::admin::registry::AdminFieldMeta) -> bool {
    if field.name == "id" || field.hidden || field.is_timestamp || field.is_foreign_key {
        return false;
    }
    field.input_type == "text"
}

fn value_as_lov_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            value.to_string().trim_matches('"').to_owned()
        }
    }
}

fn object_id(obj: &serde_json::Map<String, Value>) -> Option<String> {
    obj.get("id").map(value_as_lov_string)
}

fn map_lov_item(rec: &Value, label_field: &str) -> Option<LovItem> {
    let obj = rec.as_object()?;
    let id = object_id(obj)?;
    let label = obj
        .get(label_field)
        .map_or_else(|| id.clone(), value_as_lov_string);
    Some(LovItem { id, label })
}

fn record_matches_parent_id(obj: &serde_json::Map<String, Value>, parent_id_lower: &str) -> bool {
    obj.iter().any(|(key, value)| {
        key.ends_with("_id")
            && key != "id"
            && value_as_lov_string(value).to_lowercase() == parent_id_lower
    })
}

/// LOV endpoint — returns `[{id, label}]` JSON for FK dropdown population.
///
/// # Query parameters
///
/// - `q` — optional case-insensitive substring filter on the label field.
/// - `parent_id` — optional scoping filter; retains records where any `*_id`
///   field matches the given value.
///
/// # Errors
///
/// Returns `404 Not Found` when the entity slug is not registered.
#[rocket::get("/ui/lov/<entity_slug>?<q>&<parent_id>")]
pub async fn lov_endpoint(
    entity_slug: &str,
    q: Option<&str>,
    parent_id: Option<&str>,
    state: Option<&State<AdminState>>,
) -> Result<Json<Vec<LovItem>>, status::Custom<String>> {
    let entity = AdminRegistry::find(entity_slug).ok_or_else(|| {
        status::Custom(Status::NotFound, format!("Unknown entity: {entity_slug}"))
    })?;

    let Some(admin_state) = state else {
        return Ok(Json(Vec::new()));
    };
    let adapter = match resolve_adapter(entity_slug, admin_state.inner()) {
        Some(adapter) => adapter,
        None => return Ok(Json(Vec::new())),
    };

    let records = adapter.list_all().await.unwrap_or_default();
    let label_field = display_field(entity);

    let mut items: Vec<LovItem> = records
        .iter()
        .filter_map(|rec| map_lov_item(rec, &label_field))
        .collect();

    if let Some(pid) = parent_id.filter(|p| !p.is_empty()) {
        let pid_lower = pid.to_lowercase();
        let matching_ids = records
            .iter()
            .filter_map(|rec| {
                let obj = rec.as_object()?;
                let has_match = record_matches_parent_id(obj, &pid_lower);
                if has_match { object_id(obj) } else { None }
            })
            .collect::<std::collections::HashSet<_>>();
        items.retain(|item| matching_ids.contains(&item.id));
    }

    if let Some(query) = q.filter(|q| !q.is_empty()) {
        let q_lower = query.to_lowercase();
        items.retain(|item| item.label.to_lowercase().contains(&q_lower));
    }

    items.truncate(DEFAULT_LOV_LIMIT);

    Ok(Json(items))
}
