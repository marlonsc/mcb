//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! View-model helpers for the admin web UI.

use serde::Serialize;

use crate::admin::AdminRegistry;

/// A single entity link shown in the sidebar navigation.
#[derive(Debug, Clone, Serialize)]
pub struct NavEntityLink {
    /// URL-safe slug used for routing (e.g. `"org_entities"`).
    pub slug: String,
    /// Human-readable title shown in the nav (e.g. `"Organizations"`).
    pub title: String,
}

/// A group of related entity links in the sidebar navigation.
#[derive(Debug, Clone, Serialize)]
pub struct NavGroup {
    /// Group heading (e.g. `"Domain"`, `"Infrastructure"`).
    pub name: String,
    /// Entities belonging to this group.
    pub entities: Vec<NavEntityLink>,
}

/// Card data for a single entity on the admin dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct DashboardEntityCard {
    /// URL-safe slug.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// Group this entity belongs to.
    pub group: String,
    /// Number of schema fields.
    pub field_count: usize,
    /// Live record count from the database.
    pub record_count: usize,
}

/// Build navigation groups from the [`AdminRegistry`], sorted alphabetically.
#[must_use]
pub fn nav_groups() -> Vec<NavGroup> {
    let mut groups = std::collections::BTreeMap::<String, Vec<NavEntityLink>>::new();

    for entity in AdminRegistry::all() {
        groups
            .entry(entity.group.to_owned())
            .or_default()
            .push(NavEntityLink {
                slug: entity.slug.to_owned(),
                title: entity.title.to_owned(),
            });
    }

    groups
        .into_iter()
        .map(|(name, entities)| NavGroup { name, entities })
        .collect()
}
