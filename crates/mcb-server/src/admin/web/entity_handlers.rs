//! Entity CRUD handlers — schema-driven Handlebars template rendering.

use rocket::http::Status;
use rocket::response::status;
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
            nav_items: nav_items(),
        },
    ))
}
