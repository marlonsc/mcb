//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Entity CRUD handlers — schema-driven Handlebars template rendering.

use axum::extract::Query;
use axum::extract::{Form, Path, State};
use axum::response::Redirect;

use std::collections::HashSet;

use mcb_domain::warn;

use crate::admin::AdminRegistry;
use crate::admin::crud_adapter::resolve_adapter;
use crate::admin::error::AdminError;
use crate::admin::handlers::AdminState;
use crate::admin::web::filter::FilterParams;
use crate::admin::web::view_model::nav_groups;
use crate::templates::Template;

fn find_or_404(slug: &str) -> Result<&'static crate::admin::registry::AdminEntityMeta, AdminError> {
    AdminRegistry::find(slug)
        .ok_or_else(|| AdminError::not_found(format!("Unknown entity slug: {slug}")))
}

/// Entity catalog page — lists all registered entities with field counts.
pub async fn entities_index(State(state): State<AdminState>) -> Template {
    let mut entities = Vec::new();
    let mut total_records: usize = 0;

    for entity in AdminRegistry::all() {
        let record_count = match resolve_adapter(entity.slug, &state) {
            Some(adapter) => match adapter.list_all().await {
                Ok(v) => v.len(),
                Err(e) => {
                    warn!("AdminWeb", "list_all failed", &e);
                    0
                }
            },
            None => 0,
        };
        total_records += record_count;
        entities.push(crate::admin::web::view_model::DashboardEntityCard {
            slug: entity.slug.to_owned(),
            title: entity.title.to_owned(),
            group: entity.group.to_owned(),
            field_count: entity.fields().iter().filter(|field| !field.hidden).count(),
            record_count,
        });
    }

    let entity_count = entities.len();
    let group_count = nav_groups().len();

    Template::render(
        "admin/entity_index",
        context! {
            title: "Entities",
            current_page: "entities",
            entities: entities,
            entity_count: entity_count,
            group_count: group_count,
            total_records: total_records,
            nav_groups: nav_groups(),
        },
    )
}

/// Entity list page with filtering, sorting, and pagination.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_list(
    Path(slug): Path<String>,
    Query(params): Query<FilterParams>,
    State(state): State<AdminState>,
) -> Result<Template, AdminError> {
    let entity = find_or_404(&slug)?;

    let fields = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect::<Vec<_>>();
    let field_names = fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();
    let valid_sort_fields = field_names.iter().cloned().collect::<HashSet<_>>();

    let result = match resolve_adapter(&slug, &state) {
        Some(adapter) => adapter
            .list_filtered(&params, &valid_sort_fields)
            .await
            .unwrap_or_default(),
        None => crate::admin::web::filter::FilteredResult {
            records: Vec::new(),
            total_count: 0,
            page: params.page,
            per_page: params.per_page,
            total_pages: 0,
        },
    };
    let has_records = !result.records.is_empty();

    Ok(Template::render(
        "admin/entity_list",
        context! {
            title: entity.title,
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            fields: fields,
            field_names: field_names,
            records: result.records,
            has_records: has_records,
            total_count: result.total_count,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
            nav_groups: nav_groups(),
        },
    ))
}

/// New entity form — renders an empty form with schema-driven fields.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_new_form(Path(slug): Path<String>) -> Result<Template, AdminError> {
    let entity = find_or_404(&slug)?;

    let fields = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect::<Vec<_>>();

    Ok(Template::render(
        "admin/entity_form",
        context! {
            title: format!("New {}", entity.title),
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            fields: fields,
            is_edit: false,
            record: serde_json::Value::Object(Default::default()),
            nav_groups: nav_groups(),
        },
    ))
}

/// Detail view — fetches a single record via service adapter when available.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_detail(
    Path((slug, id)): Path<(String, String)>,
    State(state): State<AdminState>,
) -> Result<Template, AdminError> {
    let entity = find_or_404(&slug)?;
    let fields = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect::<Vec<_>>();

    let (record, has_record) = match resolve_adapter(&slug, &state) {
        Some(adapter) => match adapter.get_by_id(&id).await {
            Ok(val) => (val, true),
            Err(_) => (serde_json::Value::Null, false),
        },
        None => (serde_json::Value::Null, false),
    };

    Ok(Template::render(
        "admin/entity_detail",
        context! {
            title: entity.title,
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            fields: fields,
            record: record,
            has_record: has_record,
            nav_groups: nav_groups(),
        },
    ))
}

/// Edit form — fetches record for pre-fill via service adapter when available.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_edit_form(
    Path((slug, id)): Path<(String, String)>,
    State(state): State<AdminState>,
) -> Result<Template, AdminError> {
    let entity = find_or_404(&slug)?;
    let fields = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect::<Vec<_>>();

    let record = match resolve_adapter(&slug, &state) {
        Some(adapter) => adapter
            .get_by_id(&id)
            .await
            .unwrap_or(serde_json::Value::Null),
        None => serde_json::Value::Null,
    };

    Ok(Template::render(
        "admin/entity_form",
        context! {
            title: format!("Edit {}", entity.title),
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            fields: fields,
            is_edit: true,
            record: record,
            nav_groups: nav_groups(),
        },
    ))
}

/// Delete confirmation page.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_delete_confirm(
    Path((slug, id)): Path<(String, String)>,
    State(state): State<AdminState>,
) -> Result<Template, AdminError> {
    let entity = find_or_404(&slug)?;
    let fields = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect::<Vec<_>>();

    let (record, has_record) = match resolve_adapter(&slug, &state) {
        Some(adapter) => match adapter.get_by_id(&id).await {
            Ok(val) => (val, true),
            Err(_) => (serde_json::Value::Null, false),
        },
        None => (serde_json::Value::Null, false),
    };

    Ok(Template::render(
        "admin/entity_delete",
        context! {
            title: format!("Delete {}", entity.title),
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            fields: fields,
            record: record,
            has_record: has_record,
            nav_groups: nav_groups(),
        },
    ))
}

/// Create entity — persists via service adapter and redirects to the list page.
///
/// # Errors
/// Returns `404` for unknown entity slugs, `400` for invalid form payloads, and `500` for persistence failures.
pub async fn entities_create(
    Path(slug): Path<String>,
    State(state): State<AdminState>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, AdminError> {
    find_or_404(&slug)?;

    if let Some(adapter) = resolve_adapter(&slug, &state) {
        let data =
            serde_json::to_value(form).map_err(|e| AdminError::bad_request(e.to_string()))?;
        adapter
            .create_from_json(data)
            .await
            .map_err(AdminError::internal)?;
    }

    let url = format!("/ui/entities/{slug}?toast=created");
    Ok(Redirect::to(&url))
}

/// Update entity — persists via service adapter and redirects to the detail page.
///
/// # Errors
/// Returns `404` for unknown entity slugs, `400` for invalid form payloads, and `500` for persistence failures.
pub async fn entities_update(
    Path((slug, id)): Path<(String, String)>,
    State(state): State<AdminState>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, AdminError> {
    find_or_404(&slug)?;

    if let Some(adapter) = resolve_adapter(&slug, &state) {
        let mut map = form;
        map.insert("id".to_owned(), id.clone());
        let data = serde_json::to_value(map).map_err(|e| AdminError::bad_request(e.to_string()))?;
        adapter
            .update_from_json(data)
            .await
            .map_err(AdminError::internal)?;
    }

    let url = format!("/ui/entities/{slug}/{id}?toast=updated");
    Ok(Redirect::to(&url))
}

/// Bulk delete — deletes multiple records by comma-separated IDs.
///
/// # Errors
/// Returns `404` when entity slug is not registered.
pub async fn entities_bulk_delete(
    Path(slug): Path<String>,
    State(state): State<AdminState>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, AdminError> {
    find_or_404(&slug)?;

    let ids_raw = form.get("ids").map_or("", std::string::String::as_str);
    let ids = ids_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if let Some(adapter) = resolve_adapter(&slug, &state) {
        let total = ids.len();
        let mut failed = 0usize;
        for id in &ids {
            if adapter.delete_by_id(id).await.is_err() {
                failed += 1;
            }
        }
        let success = total - failed;
        let toast = if failed == 0 {
            format!("bulk_deleted&count={success}")
        } else if success == 0 {
            format!("bulk_error&count={failed}")
        } else {
            format!("bulk_partial&success={success}&failed={failed}")
        };
        let url = format!("/ui/entities/{slug}?toast={toast}");
        return Ok(Redirect::to(&url));
    }

    let url = format!("/ui/entities/{slug}?toast=bulk_error&count=0");
    Ok(Redirect::to(&url))
}

/// Delete entity — removes via service adapter and redirects to the list page.
///
/// # Errors
/// Returns `404` for unknown entity slugs and `500` for persistence failures.
pub async fn entities_delete(
    Path((slug, id)): Path<(String, String)>,
    State(state): State<AdminState>,
) -> Result<Redirect, AdminError> {
    find_or_404(&slug)?;

    if let Some(adapter) = resolve_adapter(&slug, &state) {
        adapter
            .delete_by_id(&id)
            .await
            .map_err(AdminError::internal)?;
    }

    let url = format!("/ui/entities/{slug}?toast=deleted");
    Ok(Redirect::to(&url))
}
