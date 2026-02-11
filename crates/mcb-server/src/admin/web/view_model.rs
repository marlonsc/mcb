use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::admin::AdminRegistry;

#[derive(Debug, Clone, Serialize)]
pub struct NavEntityLink {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavGroup {
    pub name: String,
    pub entities: Vec<NavEntityLink>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardEntityCard {
    pub slug: String,
    pub title: String,
    pub group: String,
    pub field_count: usize,
    pub record_count: usize,
    pub summary: String,
}

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

#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

#[must_use]
pub fn format_timestamp(unix_seconds: i64) -> String {
    DateTime::<Utc>::from_timestamp(unix_seconds, 0)
        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
