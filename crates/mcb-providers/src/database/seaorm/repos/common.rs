//! Common utilities and macros for `SeaORM` repository implementations.
//!
//! Provides error helpers and CRUD macros used across all repository modules.

use mcb_domain::error::Error;
use sea_orm::DbErr;

pub(crate) fn db_err(e: DbErr) -> Error {
    Error::database_with_source("Database error", e)
}

pub(crate) fn db_error(context: &str) -> impl FnOnce(DbErr) -> Error + '_ {
    move |e| Error::database_with_source(context, e)
}

// ============================================================================
// Shared auto-creation helpers
// ============================================================================

use mcb_domain::error::Result;
use mcb_utils::constants::values::DEFAULT_ORG_NAME;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ConnectionTrait, EntityTrait, Set};

use crate::database::seaorm::entities::{organization, project};

/// Ensures an organization and a project row exist (idempotent).
///
/// Uses `ON CONFLICT DO NOTHING` so concurrent calls are safe.
pub(crate) async fn ensure_org_and_project(
    db: &impl ConnectionTrait,
    org_id: &str,
    project_id: &str,
    timestamp: i64,
) -> Result<()> {
    let org = organization::ActiveModel {
        id: Set(org_id.to_owned()),
        name: Set(DEFAULT_ORG_NAME.to_owned()),
        slug: Set(org_id.to_owned()),
        settings_json: Set("{}".to_owned()),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    };
    match organization::Entity::insert(org)
        .on_conflict(
            OnConflict::column(organization::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await
    {
        Ok(_) | Err(DbErr::RecordNotInserted) => {}
        Err(other) => return Err(db_err(other)),
    }

    let proj = project::ActiveModel {
        id: Set(project_id.to_owned()),
        org_id: Set(org_id.to_owned()),
        name: Set(format!("Project {project_id}")),
        path: Set(project_id.to_owned()),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    };
    match project::Entity::insert(proj)
        .on_conflict(
            OnConflict::column(project::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await
    {
        Ok(_) | Err(DbErr::RecordNotInserted) => {}
        Err(other) => return Err(db_err(other)),
    }

    Ok(())
}

#[macro_use]
mod repo_macros;
#[macro_use]
mod trait_impl_macros;
#[macro_use]
mod pub_crud_macros;
