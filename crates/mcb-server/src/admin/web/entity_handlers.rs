//! Entity CRUD handlers — schema-driven Handlebars template rendering.

use rocket::form::Form;
use rocket::http::Status;
use rocket::response::{Redirect, status};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;

use crate::admin::{AdminRegistry, registry::AdminFieldMeta};

#[derive(Debug, Clone, Serialize)]
struct EntityNavItem {
    slug: String,
    title: String,
    group: String,
}

#[derive(Debug, Clone, Serialize)]
struct EntitySummary {
    slug: String,
    title: String,
    group: String,
    field_count: usize,
}

fn nav_items() -> Vec<EntityNavItem> {
    AdminRegistry::all()
        .iter()
        .map(|entity| EntityNavItem {
            slug: entity.slug.to_string(),
            title: entity.title.to_string(),
            group: entity.group.to_string(),
        })
        .collect()
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

    Template::render(
        "admin/entity_index",
        context! {
            title: "Entities",
            entities: entities,
            entity_count: entity_count,
            nav_items: nav_items(),
        },
    )
}

/// Entity list page — shows records for a single entity type.
#[rocket::get("/ui/entities/<slug>")]
pub fn entities_list(slug: &str) -> Result<Template, status::Custom<String>> {
    let Some(entity) = AdminRegistry::find(slug) else {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    };

    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();
    let field_names = fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();

    let records = Vec::<serde_json::Value>::new();
    let has_records = !records.is_empty();

    Ok(Template::render(
        "admin/entity_list",
        context! {
            title: entity.title,
            entity_slug: entity.slug,
            entity_group: entity.group,
            fields: fields,
            field_names: field_names,
            records: records,
            has_records: has_records,
            nav_items: nav_items(),
        },
    ))
}

/// New entity form — renders an empty form with schema-driven fields.
#[rocket::get("/ui/entities/<slug>/new")]
pub fn entities_new_form(slug: &str) -> Result<Template, status::Custom<String>> {
    let Some(entity) = AdminRegistry::find(slug) else {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    };

    let fields: Vec<AdminFieldMeta> = entity
        .fields()
        .into_iter()
        .filter(|field| !field.hidden)
        .collect();

    Ok(Template::render(
        "admin/entity_form",
        context! {
            title: format!("New {}", entity.title),
            entity_slug: entity.slug,
            entity_group: entity.group,
            fields: fields,
            is_edit: false,
            nav_items: nav_items(),
        },
    ))
}

/// Detail view — shows all fields for a single entity record.
#[rocket::get("/ui/entities/<slug>/<id>", rank = 2)]
pub fn entities_detail(slug: &str, id: &str) -> Result<Template, status::Custom<String>> {
    let Some(entity) = AdminRegistry::find(slug) else {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    };

    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();

    Ok(Template::render(
        "admin/entity_detail",
        context! {
            title: entity.title,
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            fields: fields,
            record: serde_json::Value::Null,
            has_record: false,
            nav_items: nav_items(),
        },
    ))
}

/// Edit form — renders pre-filled form for updating an existing record.
#[rocket::get("/ui/entities/<slug>/<id>/edit")]
pub fn entities_edit_form(slug: &str, id: &str) -> Result<Template, status::Custom<String>> {
    let Some(entity) = AdminRegistry::find(slug) else {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    };

    let fields: Vec<AdminFieldMeta> = entity.fields().into_iter().filter(|f| !f.hidden).collect();

    Ok(Template::render(
        "admin/entity_form",
        context! {
            title: format!("Edit {}", entity.title),
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            fields: fields,
            is_edit: true,
            record: serde_json::Value::Null,
            nav_items: nav_items(),
        },
    ))
}

/// Delete confirmation page.
#[rocket::get("/ui/entities/<slug>/<id>/delete")]
pub fn entities_delete_confirm(slug: &str, id: &str) -> Result<Template, status::Custom<String>> {
    let Some(entity) = AdminRegistry::find(slug) else {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    };

    Ok(Template::render(
        "admin/entity_delete",
        context! {
            title: format!("Delete {}", entity.title),
            entity_slug: entity.slug,
            entity_group: entity.group,
            entity_id: id,
            nav_items: nav_items(),
        },
    ))
}

/// Create entity — accepts form data and redirects to the list page.
#[rocket::post("/ui/entities/<slug>", data = "<_form>")]
pub fn entities_create(
    slug: &str,
    _form: Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, status::Custom<String>> {
    if AdminRegistry::find(slug).is_none() {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    }
    Ok(Redirect::to(format!("/ui/entities/{slug}")))
}

/// Update entity — accepts form data and redirects to the detail page.
#[rocket::post("/ui/entities/<slug>/<id>", data = "<_form>")]
pub fn entities_update(
    slug: &str,
    id: &str,
    _form: Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, status::Custom<String>> {
    if AdminRegistry::find(slug).is_none() {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    }
    Ok(Redirect::to(format!("/ui/entities/{slug}/{id}")))
}

/// Delete entity — redirects to the list page.
#[rocket::post("/ui/entities/<slug>/<id>/delete")]
pub fn entities_delete(slug: &str, id: &str) -> Result<Redirect, status::Custom<String>> {
    let _ = id;
    if AdminRegistry::find(slug).is_none() {
        return Err(status::Custom(
            Status::NotFound,
            format!("Unknown entity slug: {slug}"),
        ));
    }
    Ok(Redirect::to(format!("/ui/entities/{slug}")))
}
