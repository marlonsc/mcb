//! View-model helpers for the admin web UI.

use chrono::{DateTime, Utc};
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
    /// Short description of the entity.
    pub summary: String,
}

/// Build navigation groups from the [`AdminRegistry`], sorted alphabetically.
#[must_use]
pub fn nav_groups() -> Vec<NavGroup> {
    let mut groups = std::collections::BTreeMap::<String, Vec<NavEntityLink>>::new();

    for entity in AdminRegistry::all() {
        groups
            .entry(entity.group.to_string())
            .or_default()
            .push(NavEntityLink {
                slug: entity.slug.to_string(),
                title: entity.title.to_string(),
            });
    }

    groups
        .into_iter()
        .map(|(name, entities)| NavGroup { name, entities })
        .collect()
}

/// Truncate `input` to `max_len` characters, appending `…` when shortened.
#[must_use]
pub fn truncate_text(input: &str, max_len: usize) -> String {
    if input.chars().count() <= max_len {
        return input.to_string();
    }

    let mut out = input
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

/// Return `singular` when `count == 1`, otherwise `plural`.
#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

/// Format a Unix-epoch timestamp as `YYYY-MM-DD HH:MM:SS UTC`.
#[must_use]
pub fn format_timestamp(unix_seconds: i64) -> String {
    DateTime::<Utc>::from_timestamp(unix_seconds, 0)
        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
