//! Admin panel configuration and dashboard endpoints.
//!
//! Serves sea-orm-pro TOML configs as JSON for the admin frontend,
//! plus MCB-specific dashboard queries (observation counts, session stats).

use axum::http::HeaderMap;
use loco_rs::prelude::*;
use sea_orm::{
    DatabaseBackend, DbConn, DeriveColumn, EnumIter, FromQueryResult, QueryOrder, QuerySelect,
    sea_query::{Asterisk, Expr, Func},
};
use sea_orm_pro::ConfigParser;

use serde::{Deserialize, Serialize};

use mcb_providers::database::seaorm::entities::{agent_sessions, observations, tool_calls};

use crate::constants::admin::CONFIG_ROOT;

/// Returns the sea-orm-pro admin panel configuration as JSON.
///
/// In development the config reloads from disk on every request so edits
/// are reflected immediately.
///
/// # Errors
///
/// Returns an error if the config files under `config/pro_admin/` cannot be parsed.
pub async fn config(State(ctx): State<AppContext>, headers: HeaderMap) -> Result<Response> {
    crate::auth::authorize_admin_api_key(&ctx, &headers).await?;
    let config = ConfigParser::new()
        .load_config(CONFIG_ROOT)
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
    format::json(config)
}

/// Request body for the `/admin/dashboard` endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardBody {
    /// Dashboard graph identifier (e.g. `"observations_by_month"`).
    pub graph: String,
    /// Optional start of the date range filter.
    pub from: Option<sea_orm::prelude::DateTime>,
    /// Optional end of the date range filter.
    pub to: Option<sea_orm::prelude::DateTime>,
}

/// A single key/value data-point returned by dashboard queries.
#[derive(Debug, Deserialize, Serialize, FromQueryResult, PartialEq)]
pub struct Datum {
    /// Grouping label (date bucket, tool name, etc.).
    pub key: String,
    /// Aggregated count for the group.
    pub val: i32,
}

/// `SeaORM` column selector for [`Datum`] projections.
#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum DatumColumn {
    /// Maps to [`Datum::key`].
    Key,
    /// Maps to [`Datum::val`].
    Val,
}

/// Executes a named dashboard query and returns the result as JSON.
///
/// Supported graphs: `observations_by_month`, `sessions_by_day`,
/// `tool_calls_by_tool`.
///
/// # Errors
///
/// Returns an error if the database query fails or the graph name is unknown.
pub async fn dashboard(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Json(body): Json<DashboardBody>,
) -> Result<Response> {
    crate::auth::authorize_admin_api_key(&ctx, &headers).await?;
    let db = &ctx.db;
    let data = match body.graph.as_str() {
        "observations_by_month" => {
            observations::Entity::find()
                .select_only()
                .column_as(
                    cast_as_year_month(db, observations::Column::CreatedAt),
                    DatumColumn::Key,
                )
                .column_as(
                    Expr::expr(Func::cast_as(
                        Func::count(Expr::col(Asterisk)),
                        int_keyword(db),
                    )),
                    DatumColumn::Val,
                )
                .group_by(Expr::col(DatumColumn::Key))
                .order_by_asc(Expr::col(DatumColumn::Key))
                .into_model::<Datum>()
                .all(db)
                .await?
        }
        "sessions_by_day" => {
            agent_sessions::Entity::find()
                .select_only()
                .column_as(
                    cast_as_day(db, agent_sessions::Column::StartedAt),
                    DatumColumn::Key,
                )
                .column_as(
                    Expr::expr(Func::cast_as(
                        Func::count(Expr::col(Asterisk)),
                        int_keyword(db),
                    )),
                    DatumColumn::Val,
                )
                .group_by(Expr::col(DatumColumn::Key))
                .order_by_asc(Expr::col(DatumColumn::Key))
                .into_model::<Datum>()
                .all(db)
                .await?
        }
        "tool_calls_by_tool" => {
            tool_calls::Entity::find()
                .select_only()
                .column_as(Expr::col(tool_calls::Column::ToolName), DatumColumn::Key)
                .column_as(
                    Expr::expr(Func::cast_as(
                        Func::count(Expr::col(Asterisk)),
                        int_keyword(db),
                    )),
                    DatumColumn::Val,
                )
                .group_by(Expr::col(DatumColumn::Key))
                .order_by_desc(Expr::col(DatumColumn::Val))
                .into_model::<Datum>()
                .all(db)
                .await?
        }
        _ => not_found()?,
    };
    format::json(data)
}

fn cast_as_year_month(db: &DbConn, col: impl sea_orm::sea_query::IntoColumnRef) -> Expr {
    let col_ref = col.into_column_ref();
    let func = match db.get_database_backend() {
        DatabaseBackend::MySql => Func::cust(sea_orm::sea_query::Alias::new("DATE_FORMAT"))
            .arg(Expr::expr(
                Func::cust("FROM_UNIXTIME").arg(Expr::col(col_ref.clone())),
            ))
            .arg("%Y-%m"),
        DatabaseBackend::Postgres => Func::cust(sea_orm::sea_query::Alias::new("TO_CHAR"))
            .arg(Expr::expr(
                Func::cust("to_timestamp").arg(Expr::col(col_ref)),
            ))
            .arg("YYYY-MM"),
        // i64 Unix timestamps require the 'unixepoch' modifier for SQLite STRFTIME.
        DatabaseBackend::Sqlite | _ => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m")
            .arg(Expr::col(col_ref))
            .arg("unixepoch"),
    };
    Expr::expr(func)
}

fn cast_as_day(db: &DbConn, col: impl sea_orm::sea_query::IntoColumnRef) -> Expr {
    let col_ref = col.into_column_ref();
    let func = match db.get_database_backend() {
        DatabaseBackend::MySql => Func::cust(sea_orm::sea_query::Alias::new("DATE_FORMAT"))
            .arg(Expr::expr(
                Func::cust("FROM_UNIXTIME").arg(Expr::col(col_ref.clone())),
            ))
            .arg("%Y-%m-%d"),
        DatabaseBackend::Postgres => Func::cust(sea_orm::sea_query::Alias::new("TO_CHAR"))
            .arg(Expr::expr(
                Func::cust("to_timestamp").arg(Expr::col(col_ref)),
            ))
            .arg("YYYY-MM-DD"),
        // i64 Unix timestamps require the 'unixepoch' modifier for SQLite STRFTIME.
        DatabaseBackend::Sqlite | _ => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m-%d")
            .arg(Expr::col(col_ref))
            .arg("unixepoch"),
    };
    Expr::expr(func)
}

fn int_keyword(db: &DbConn) -> impl sea_orm::sea_query::IntoIden {
    match db.get_database_backend() {
        DatabaseBackend::MySql => sea_orm::sea_query::Alias::new("SIGNED INTEGER"),
        DatabaseBackend::Postgres => sea_orm::sea_query::Alias::new("INT4"),
        DatabaseBackend::Sqlite | _ => sea_orm::sea_query::Alias::new("INT"),
    }
}

/// Registers `/admin/config` and `/admin/dashboard` routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/config", get(config))
        .add("/dashboard", post(dashboard))
}
