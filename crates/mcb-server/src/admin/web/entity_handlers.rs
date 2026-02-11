//! Entity CRUD handlers — schema-driven Handlebars template rendering.

use crate::context;
use crate::templates::Template;
use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::{Redirect, status};
use serde::Serialize;

use crate::admin::crud_adapter::resolve_adapter;
use crate::admin::handlers::AdminState;
use crate::admin::web::view_model::nav_groups;
use crate::admin::{AdminRegistry, registry::AdminFieldMeta};

#[derive(Debug, Clone, Serialize)]
struct EntitySummary {
    slug: String,
    title: String,
    group: String,
    field_count: usize,
}

fn find_or_404(
    slug: &str,
) -> Result<&'static crate::admin::registry::AdminEntityMeta, status::Custom<String>> {
    AdminRegistry::find(slug)
        .ok_or_else(|| status::Custom(Status::NotFound, format!("Unknown entity slug: {slug}")))
}

/// Entity catalog page — lists all registered entities with field counts.
#[rocket::get("/ui/entities")]
pub fn entities_index() -> Template {
    let entities = AdminRegistry::all()
        .iter()
        .map(|entity| EntitySummary {
            slug: entity.slug.to_string(),
            title: entity.title.to_string(),
            group: entity.group.to_string(),
            field_count: entity.fields().iter().filter(|field| !field.hidden).count(),
        })
        .collect::<Vec<_>>();

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
            nav_groups: nav_groups(),
        },
    )
}

/// Entity list page — fetches records via service adapter when available.
#[rocket::get("/ui/entities/<slug>")]
pub async fn entities_list(
    slug: &str,
    state: Option<&State<AdminState>>,
) -> Result<Template, status::Custom<String>> {
    let entity = find_or_404(slug)?;

    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();
    let field_names = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

    let records = match state.and_then(|s| resolve_adapter(slug, s.inner())) {
        Some(adapter) => adapter.list_all().await.unwrap_or_default(),
        None => Vec::new(),
    };
    let has_records = !records.is_empty();

    Ok(Template::render(
        "admin/entity_list",
        context! {
            title: entity.title,
            current_page: entity.slug,
            entity_slug: entity.slug,
            entity_group: entity.group,
            fields: fields,
            field_names: field_names,
            records: records,
            has_records: has_records,
            nav_groups: nav_groups(),
        },
    ))
}

/// New entity form — renders an empty form with schema-driven fields.
#[rocket::get("/ui/entities/<slug>/new")]
pub fn entities_new_form(slug: &str) -> Result<Template, status::Custom<String>> {
    let entity = find_or_404(slug)?;

    let fields: Vec<AdminFieldMeta> = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect();

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
#[rocket::get("/ui/entities/<slug>/<id>", rank = 2)]
pub async fn entities_detail(
    slug: &str,
    id: &str,
    state: Option<&State<AdminState>>,
) -> Result<Template, status::Custom<String>> {
    let entity = find_or_404(slug)?;
    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();

    let (record, has_record) = match state.and_then(|s| resolve_adapter(slug, s.inner())) {
        Some(adapter) => match adapter.get_by_id(id).await {
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
#[rocket::get("/ui/entities/<slug>/<id>/edit")]
pub async fn entities_edit_form(
    slug: &str,
    id: &str,
    state: Option<&State<AdminState>>,
) -> Result<Template, status::Custom<String>> {
    let entity = find_or_404(slug)?;
    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();

    let record = match state.and_then(|s| resolve_adapter(slug, s.inner())) {
        Some(adapter) => adapter
            .get_by_id(id)
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
#[rocket::get("/ui/entities/<slug>/<id>/delete")]
pub async fn entities_delete_confirm(
    slug: &str,
    id: &str,
    state: Option<&State<AdminState>>,
) -> Result<Template, status::Custom<String>> {
    let entity = find_or_404(slug)?;
    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();

    let (record, has_record) = match state.and_then(|s| resolve_adapter(slug, s.inner())) {
        Some(adapter) => match adapter.get_by_id(id).await {
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
#[rocket::post("/ui/entities/<slug>", data = "<form>")]
pub async fn entities_create(
    slug: &str,
    form: Form<std::collections::HashMap<String, String>>,
    state: Option<&State<AdminState>>,
) -> Result<Redirect, status::Custom<String>> {
    find_or_404(slug)?;

    if let Some(adapter) = state.and_then(|s| resolve_adapter(slug, s.inner())) {
        let data = serde_json::to_value(form.into_inner())
            .map_err(|e| status::Custom(Status::BadRequest, e.to_string()))?;
        adapter
            .create_from_json(data)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e))?;
    }

    Ok(Redirect::to(format!("/ui/entities/{slug}")))
}

/// Update entity — persists via service adapter and redirects to the detail page.
#[rocket::post("/ui/entities/<slug>/<id>", data = "<form>")]
pub async fn entities_update(
    slug: &str,
    id: &str,
    form: Form<std::collections::HashMap<String, String>>,
    state: Option<&State<AdminState>>,
) -> Result<Redirect, status::Custom<String>> {
    find_or_404(slug)?;

    if let Some(adapter) = state.and_then(|s| resolve_adapter(slug, s.inner())) {
        let mut map = form.into_inner();
        map.insert("id".to_string(), id.to_string());
        let data = serde_json::to_value(map)
            .map_err(|e| status::Custom(Status::BadRequest, e.to_string()))?;
        adapter
            .update_from_json(data)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e))?;
    }

    Ok(Redirect::to(format!("/ui/entities/{slug}/{id}")))
}

/// Delete entity — removes via service adapter and redirects to the list page.
#[rocket::post("/ui/entities/<slug>/<id>/delete")]
pub async fn entities_delete(
    slug: &str,
    id: &str,
    state: Option<&State<AdminState>>,
) -> Result<Redirect, status::Custom<String>> {
    find_or_404(slug)?;

    if let Some(adapter) = state.and_then(|s| resolve_adapter(slug, s.inner())) {
        adapter
            .delete_by_id(id)
            .await
            .map_err(|e| status::Custom(Status::InternalServerError, e))?;
    }

    Ok(Redirect::to(format!("/ui/entities/{slug}")))
}
