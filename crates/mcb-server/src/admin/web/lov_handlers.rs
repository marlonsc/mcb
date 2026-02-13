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

/// Maximum number of LOV items returned per request.
const LOV_LIMIT: usize = 50;

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
        .find(|f| {
            f.name != "id"
                && !f.hidden
                && !f.is_timestamp
                && !f.is_foreign_key
                && f.input_type == "text"
        })
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "id".to_string())
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

    let adapter = match state.and_then(|s| resolve_adapter(entity_slug, s.inner())) {
        Some(a) => a,
        None => return Ok(Json(Vec::new())),
    };

    let records = adapter.list_all().await.unwrap_or_default();
    let label_field = display_field(entity);

    let mut items: Vec<LovItem> = records
        .iter()
        .filter_map(|rec| {
            let obj = rec.as_object()?;

            let id = match obj.get("id") {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string().trim_matches('"').to_string(),
                None => return None,
            };

            let label = match obj.get(&label_field) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string().trim_matches('"').to_string(),
                None => id.clone(),
            };

            Some(LovItem { id, label })
        })
        .collect();

    if let Some(pid) = parent_id.filter(|p| !p.is_empty()) {
        let pid_lower = pid.to_lowercase();
        let matching_ids: std::collections::HashSet<String> = records
            .iter()
            .filter_map(|rec| {
                let obj = rec.as_object()?;
                let has_match = obj.iter().any(|(k, v)| {
                    k.ends_with("_id")
                        && k != "id"
                        && match v {
                            Value::String(s) => s.to_lowercase() == pid_lower,
                            _ => v.to_string().trim_matches('"').to_lowercase() == pid_lower,
                        }
                });
                if has_match {
                    match obj.get("id") {
                        Some(Value::String(s)) => Some(s.clone()),
                        Some(v) => Some(v.to_string().trim_matches('"').to_string()),
                        None => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        items.retain(|item| matching_ids.contains(&item.id));
    }

    if let Some(query) = q.filter(|q| !q.is_empty()) {
        let q_lower = query.to_lowercase();
        items.retain(|item| item.label.to_lowercase().contains(&q_lower));
    }

    items.truncate(LOV_LIMIT);

    Ok(Json(items))
}
