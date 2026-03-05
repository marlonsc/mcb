use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    AgentSessionStats, DailyCount, DashboardQueryPort, MonthlyCount, ToolCallCount,
};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, QueryResult, Statement, Value,
};

/// SeaORM-backed implementation of [`DashboardQueryPort`].
pub struct SeaOrmDashboardAdapter {
    db: DatabaseConnection,
}

impl SeaOrmDashboardAdapter {
    /// Creates an adapter using the given database connection.
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn db_err<E>(context: &str, source: E) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::database_with_source(context, source)
    }

    fn year_month_expr(&self) -> &'static str {
        match self.db.get_database_backend() {
            DatabaseBackend::MySql => "DATE_FORMAT(FROM_UNIXTIME(created_at), '%Y-%m')",
            DatabaseBackend::Postgres => "TO_CHAR(to_timestamp(created_at), 'YYYY-MM')",
            DatabaseBackend::Sqlite | _ => "STRFTIME('%Y-%m', created_at, 'unixepoch')",
        }
    }

    fn day_expr(&self) -> &'static str {
        match self.db.get_database_backend() {
            DatabaseBackend::MySql => "DATE_FORMAT(FROM_UNIXTIME(created_at), '%Y-%m-%d')",
            DatabaseBackend::Postgres => "TO_CHAR(to_timestamp(created_at), 'YYYY-MM-DD')",
            DatabaseBackend::Sqlite | _ => "STRFTIME('%Y-%m-%d', created_at, 'unixepoch')",
        }
    }

    fn cast_count_expr(&self, expr: &str) -> String {
        match self.db.get_database_backend() {
            DatabaseBackend::MySql => format!("CAST({expr} AS SIGNED INTEGER)"),
            DatabaseBackend::Postgres => format!("CAST({expr} AS BIGINT)"),
            DatabaseBackend::Sqlite | _ => format!("CAST({expr} AS INTEGER)"),
        }
    }

    fn decode_count(row: &QueryResult, column: &str) -> std::result::Result<i64, sea_orm::DbErr> {
        row.try_get::<i64>("", column)
            .or_else(|_| row.try_get::<i32>("", column).map(i64::from))
    }
    /// Shared helper for time-bucketed observation queries.
    async fn query_observation_time_series(
        &self,
        time_expr: &str,
        alias: &str,
        limit: usize,
    ) -> Result<Vec<(String, i64)>> {
        let count_expr = self.cast_count_expr("COUNT(*)");
        let sql = format!(
            "SELECT {time_expr} AS {alias}, {count_expr} AS count \
             FROM observations \
             GROUP BY {time_expr} \
             ORDER BY {alias} DESC \
             LIMIT ?"
        );
        let stmt = Statement::from_sql_and_values(
            self.db.get_database_backend(),
            sql,
            vec![Value::from(limit as i64)],
        );

        let rows = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_err(&format!("query observations by {alias}"), e))?;

        rows.into_iter()
            .map(|row| {
                let label = row
                    .try_get::<String>("", alias)
                    .map_err(|e| Self::db_err(&format!("decode {alias}"), e))?;
                let count = Self::decode_count(&row, "count")
                    .map_err(|e| Self::db_err(&format!("decode {alias} count"), e))?;
                Ok((label, count))
            })
            .collect()
    }
}
#[async_trait]
impl DashboardQueryPort for SeaOrmDashboardAdapter {
    async fn get_observations_by_month(&self, limit: usize) -> Result<Vec<MonthlyCount>> {
        self.query_observation_time_series(self.year_month_expr(), "month", limit)
            .await?
            .into_iter()
            .map(|(month, count)| Ok(MonthlyCount { month, count }))
            .collect()
    }

    async fn get_observations_by_day(&self, limit: usize) -> Result<Vec<DailyCount>> {
        self.query_observation_time_series(self.day_expr(), "day", limit)
            .await?
            .into_iter()
            .map(|(day, count)| Ok(DailyCount { day, count }))
            .collect()
    }

    async fn get_tool_call_counts(&self) -> Result<Vec<ToolCallCount>> {
        let count_expr = self.cast_count_expr("COUNT(*)");
        let sql = format!(
            "SELECT tool_name, {count_expr} AS count \
             FROM tool_calls \
             GROUP BY tool_name \
             ORDER BY count DESC"
        );
        let stmt = Statement::from_sql_and_values(self.db.get_database_backend(), sql, Vec::new());

        let rows = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_err("query tool call counts", e))?;

        rows.into_iter()
            .map(|row| {
                let tool_name = row
                    .try_get::<String>("", "tool_name")
                    .map_err(|e| Self::db_err("decode tool name", e))?;
                let count = Self::decode_count(&row, "count")
                    .map_err(|e| Self::db_err("decode tool count", e))?;
                Ok(ToolCallCount { tool_name, count })
            })
            .collect()
    }

    async fn get_agent_session_stats(&self) -> Result<AgentSessionStats> {
        let total_sessions_expr = self.cast_count_expr("COUNT(*)");
        let total_agents_expr = self.cast_count_expr("COUNT(DISTINCT agent_type)");
        let sql = format!(
            "SELECT {total_sessions_expr} AS total_sessions, \
             {total_agents_expr} AS total_agents \
             FROM agent_sessions"
        );
        let stmt = Statement::from_sql_and_values(self.db.get_database_backend(), sql, Vec::new());

        let row = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_err("query agent session stats", e))?
            .into_iter()
            .next()
            .ok_or_else(|| Error::database("missing agent session stats row"))?;

        let total_sessions = Self::decode_count(&row, "total_sessions")
            .map_err(|e| Self::db_err("decode total sessions", e))?;
        let total_agents = Self::decode_count(&row, "total_agents")
            .map_err(|e| Self::db_err("decode total agents", e))?;

        Ok(AgentSessionStats {
            total_sessions,
            total_agents,
        })
    }
}
