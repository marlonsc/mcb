//! Admin panel configuration and dashboard endpoints.
//!
//! Serves sea-orm-pro TOML configs as JSON for the admin frontend,
//! plus MCB-specific dashboard queries (observation counts, session stats).

use loco_rs::{environment::Environment, prelude::*};
use sea_orm::{
    DatabaseBackend, DbConn, DeriveColumn, EnumIter, FromQueryResult, QueryOrder, QuerySelect,
    sea_query::{Asterisk, Expr, Func},
};
use sea_orm_pro::{ConfigParser, JsonCfg};
use seaography::lazy_static;
use serde::{Deserialize, Serialize};

use mcb_providers::database::seaorm::entities::{agent_sessions, observations, tool_calls};

const CONFIG_ROOT: &str = "config/pro_admin";

lazy_static::lazy_static! {
    static ref CONFIG: JsonCfg = ConfigParser::new().load_config(CONFIG_ROOT).unwrap();
}

/// Returns the sea-orm-pro admin panel configuration as JSON.
///
/// In production the config is cached via `lazy_static`; in development it
/// reloads from disk on every request so edits are reflected immediately.
pub async fn config(State(ctx): State<AppContext>) -> Result<Response> {
    if ctx.environment == Environment::Production {
        format::json(&*CONFIG)
    } else {
        let config = ConfigParser::new()
            .load_config(CONFIG_ROOT)
            .map_err(Into::<Box<dyn std::error::Error + Send + Sync>>::into)?;
        format::json(config)
    }
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

/// SeaORM column selector for [`Datum`] projections.
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
pub async fn dashboard(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(body): Json<DashboardBody>,
) -> Result<Response> {
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
    let func = match db.get_database_backend() {
        DatabaseBackend::MySql => Func::cust(sea_orm::sea_query::Alias::new("DATE_FORMAT"))
            .arg(Expr::col(col.into_column_ref()))
            .arg("%Y-%m"),
        DatabaseBackend::Postgres => Func::cust(sea_orm::sea_query::Alias::new("TO_CHAR"))
            .arg(Expr::col(col.into_column_ref()))
            .arg("YYYY-mm"),
        DatabaseBackend::Sqlite => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m")
            .arg(Expr::col(col.into_column_ref())),
        _ => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m")
            .arg(Expr::col(col.into_column_ref())),
    };
    Expr::expr(func)
}

fn cast_as_day(db: &DbConn, col: impl sea_orm::sea_query::IntoColumnRef) -> Expr {
    let func = match db.get_database_backend() {
        DatabaseBackend::MySql => Func::cust(sea_orm::sea_query::Alias::new("DATE_FORMAT"))
            .arg(Expr::col(col.into_column_ref()))
            .arg("%Y-%m-%d"),
        DatabaseBackend::Postgres => Func::cust(sea_orm::sea_query::Alias::new("TO_CHAR"))
            .arg(Expr::col(col.into_column_ref()))
            .arg("YYYY-mm-dd"),
        DatabaseBackend::Sqlite => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m-%d")
            .arg(Expr::col(col.into_column_ref())),
        _ => Func::cust(sea_orm::sea_query::Alias::new("STRFTIME"))
            .arg("%Y-%m-%d")
            .arg(Expr::col(col.into_column_ref())),
    };
    Expr::expr(func)
}

fn int_keyword(db: &DbConn) -> impl sea_orm::sea_query::IntoIden {
    match db.get_database_backend() {
        DatabaseBackend::MySql => sea_orm::sea_query::Alias::new("SIGNED INTEGER"),
        DatabaseBackend::Postgres => sea_orm::sea_query::Alias::new("INT4"),
        DatabaseBackend::Sqlite => sea_orm::sea_query::Alias::new("INT"),
        _ => sea_orm::sea_query::Alias::new("INT"),
    }
}

/// Registers `/admin/config` and `/admin/dashboard` routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/config", get(config))
        .add("/dashboard", post(dashboard))
}
