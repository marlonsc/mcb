//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Embedded Handlebars templates compiled into the binary.
//!
//! When the template directory is unavailable at runtime (e.g., installed binary
//! without accompanying template files), these embedded templates are used as a
//! fallback. This eliminates the disk dependency for production deployments.
//!
//! Template names follow the same convention as disk-loaded templates:
//! `admin/dashboard`, `admin/entity_list`, `admin/partials/field_input`, etc.

use handlebars::Handlebars;

/// All embedded template entries: `(name, content)` pairs.
///
/// Names match the disk-loaded convention (directory structure without the
/// `.html.hbs` extension chain).
pub(crate) const EMBEDDED_TEMPLATES: &[(&str, &str)] = &[
    // ── Page templates ──────────────────────────────────────────────────
    (
        "admin/browse",
        include_str!("../../templates/admin/browse.html.hbs"),
    ),
    (
        "admin/browse_collection",
        include_str!("../../templates/admin/browse_collection.html.hbs"),
    ),
    (
        "admin/browse_file",
        include_str!("../../templates/admin/browse_file.html.hbs"),
    ),
    (
        "admin/browse_tree",
        include_str!("../../templates/admin/browse_tree.html.hbs"),
    ),
    (
        "admin/config",
        include_str!("../../templates/admin/config.html.hbs"),
    ),
    (
        "admin/dashboard",
        include_str!("../../templates/admin/dashboard.html.hbs"),
    ),
    (
        "admin/entity_delete",
        include_str!("../../templates/admin/entity_delete.html.hbs"),
    ),
    (
        "admin/entity_detail",
        include_str!("../../templates/admin/entity_detail.html.hbs"),
    ),
    (
        "admin/entity_form",
        include_str!("../../templates/admin/entity_form.html.hbs"),
    ),
    (
        "admin/entity_index",
        include_str!("../../templates/admin/entity_index.html.hbs"),
    ),
    (
        "admin/entity_list",
        include_str!("../../templates/admin/entity_list.html.hbs"),
    ),
    (
        "admin/health",
        include_str!("../../templates/admin/health.html.hbs"),
    ),
    (
        "admin/jobs",
        include_str!("../../templates/admin/jobs.html.hbs"),
    ),
    (
        "admin/layout",
        include_str!("../../templates/admin/layout.html.hbs"),
    ),
    // ── Partial templates ───────────────────────────────────────────────
    (
        "admin/partials/alpine_init",
        include_str!("../../templates/admin/partials/alpine_init.html.hbs"),
    ),
    (
        "admin/partials/bulk_actions",
        include_str!("../../templates/admin/partials/bulk_actions.html.hbs"),
    ),
    (
        "admin/partials/cascading_select",
        include_str!("../../templates/admin/partials/cascading_select.html.hbs"),
    ),
    (
        "admin/partials/confirm_dialog",
        include_str!("../../templates/admin/partials/confirm_dialog.html.hbs"),
    ),
    (
        "admin/partials/date_range",
        include_str!("../../templates/admin/partials/date_range.html.hbs"),
    ),
    (
        "admin/partials/field_input",
        include_str!("../../templates/admin/partials/field_input.html.hbs"),
    ),
    (
        "admin/partials/json_editor",
        include_str!("../../templates/admin/partials/json_editor.html.hbs"),
    ),
    (
        "admin/partials/lov_select",
        include_str!("../../templates/admin/partials/lov_select.html.hbs"),
    ),
    (
        "admin/partials/multi_select",
        include_str!("../../templates/admin/partials/multi_select.html.hbs"),
    ),
    (
        "admin/partials/pagination",
        include_str!("../../templates/admin/partials/pagination.html.hbs"),
    ),
    (
        "admin/partials/search_bar",
        include_str!("../../templates/admin/partials/search_bar.html.hbs"),
    ),
    (
        "admin/partials/sortable_header",
        include_str!("../../templates/admin/partials/sortable_header.html.hbs"),
    ),
    (
        "admin/partials/toast",
        include_str!("../../templates/admin/partials/toast.html.hbs"),
    ),
    (
        "admin/partials/toggle_switch",
        include_str!("../../templates/admin/partials/toggle_switch.html.hbs"),
    ),
];

/// Register all embedded templates into a Handlebars engine instance.
///
/// Returns `true` if all templates registered successfully, `false` if any failed.
pub(crate) fn register_embedded(hb: &mut Handlebars<'static>) -> bool {
    let mut ok = true;
    for &(name, content) in EMBEDDED_TEMPLATES {
        if let Err(e) = hb.register_template_string(name, content) {
            error!("Embedded template '{}' failed to register: {}", name, e);
            ok = false;
        }
    }
    ok
}
