//!
//! SeaORM entity for `organizations`. Generated from schema; keep in sync with
//! `mcb_domain::schema::organizations::table()`.

use sea_orm::entity::prelude::*;

crate::sea_entity!(
    "organizations",
    [
        (id: String),
        (name: String),
        (slug: String),
        (settings_json: String),
        (created_at: i64),
        (updated_at: i64),
    ]
);
