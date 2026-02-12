use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::{
    ApiKey, Organization, Team, TeamMember, TeamMemberRole, User, UserRole,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::OrgEntityRepository;

/// SQLite-backed repository for organization, user, team, and API key entities.
pub struct SqliteOrgEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteOrgEntityRepository {
    /// Creates a new repository using the provided database executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

    async fn query_all<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Vec<T>>
    where
        F: Fn(&dyn SqlRow) -> Result<T>,
    {
        let rows = self.executor.query_all(sql, params).await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(
                convert(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode org entity", e))?,
            );
        }
        Ok(result)
    }
}

fn row_to_org(row: &dyn SqlRow) -> Result<Organization> {
    Ok(Organization {
        id: req_str(row, "id")?,
        name: req_str(row, "name")?,
        slug: req_str(row, "slug")?,
        settings_json: req_str(row, "settings_json")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

fn row_to_user(row: &dyn SqlRow) -> Result<User> {
    Ok(User {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        email: req_str(row, "email")?,
        display_name: req_str(row, "display_name")?,
        role: req_str(row, "role")?
            .parse::<UserRole>()
            .map_err(|e| Error::memory(format!("Invalid user role: {e}")))?,
        api_key_hash: opt_str(row, "api_key_hash")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

fn row_to_team(row: &dyn SqlRow) -> Result<Team> {
    Ok(Team {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        name: req_str(row, "name")?,
        created_at: req_i64(row, "created_at")?,
    })
}

fn row_to_team_member(row: &dyn SqlRow) -> Result<TeamMember> {
    Ok(TeamMember {
        team_id: req_str(row, "team_id")?,
        user_id: req_str(row, "user_id")?,
        role: req_str(row, "role")?
            .parse::<TeamMemberRole>()
            .map_err(|e| Error::memory(format!("Invalid team member role: {e}")))?,
        joined_at: req_i64(row, "joined_at")?,
    })
}

fn row_to_api_key(row: &dyn SqlRow) -> Result<ApiKey> {
    Ok(ApiKey {
        id: req_str(row, "id")?,
        user_id: req_str(row, "user_id")?,
        org_id: req_str(row, "org_id")?,
        key_hash: req_str(row, "key_hash")?,
        name: req_str(row, "name")?,
        scopes_json: req_str(row, "scopes_json")?,
        expires_at: opt_i64(row, "expires_at")?,
        created_at: req_i64(row, "created_at")?,
        revoked_at: opt_i64(row, "revoked_at")?,
    })
}

fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
    row.try_get_string(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
    row.try_get_i64(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn opt_str(row: &dyn SqlRow, col: &str) -> Result<Option<String>> {
    row.try_get_string(col)
}

fn opt_i64(row: &dyn SqlRow, col: &str) -> Result<Option<i64>> {
    row.try_get_i64(col)
}

fn opt_str_param(value: &Option<String>) -> SqlParam {
    match value {
        Some(v) => SqlParam::String(v.clone()),
        None => SqlParam::Null,
    }
}

fn opt_i64_param(value: Option<i64>) -> SqlParam {
    match value {
        Some(v) => SqlParam::I64(v),
        None => SqlParam::Null,
    }
}

#[async_trait]
impl OrgEntityRepository for SqliteOrgEntityRepository {
    async fn create_org(&self, org: &Organization) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(org.id.clone()),
                    SqlParam::String(org.name.clone()),
                    SqlParam::String(org.slug.clone()),
                    SqlParam::String(org.settings_json.clone()),
                    SqlParam::I64(org.created_at),
                    SqlParam::I64(org.updated_at),
                ],
            )
            .await
    }

    async fn get_org(&self, id: &str) -> Result<Organization> {
        self.query_one(
            "SELECT * FROM organizations WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_org,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Organization {id}")))
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        self.query_all("SELECT * FROM organizations", &[], row_to_org)
            .await
    }

    async fn update_org(&self, org: &Organization) -> Result<()> {
        self.executor
            .execute(
                "UPDATE organizations SET name = ?, slug = ?, settings_json = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(org.name.clone()),
                    SqlParam::String(org.slug.clone()),
                    SqlParam::String(org.settings_json.clone()),
                    SqlParam::I64(org.updated_at),
                    SqlParam::String(org.id.clone()),
                ],
            )
            .await
    }

    async fn delete_org(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM organizations WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn create_user(&self, user: &User) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO users (id, org_id, email, display_name, role, api_key_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(user.id.clone()),
                    SqlParam::String(user.org_id.clone()),
                    SqlParam::String(user.email.clone()),
                    SqlParam::String(user.display_name.clone()),
                    SqlParam::String(user.role.as_str().to_string()),
                    opt_str_param(&user.api_key_hash),
                    SqlParam::I64(user.created_at),
                    SqlParam::I64(user.updated_at),
                ],
            )
            .await
    }

    async fn get_user(&self, id: &str) -> Result<User> {
        self.query_one(
            "SELECT * FROM users WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_user,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("User {id}")))
    }

    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        self.query_one(
            "SELECT * FROM users WHERE org_id = ? AND email = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(email.to_string()),
            ],
            row_to_user,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("User {email}")))
    }

    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        self.query_all(
            "SELECT * FROM users WHERE org_id = ?",
            &[SqlParam::String(org_id.to_string())],
            row_to_user,
        )
        .await
    }

    async fn update_user(&self, user: &User) -> Result<()> {
        self.executor
            .execute(
                "UPDATE users SET org_id = ?, email = ?, display_name = ?, role = ?, api_key_hash = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(user.org_id.clone()),
                    SqlParam::String(user.email.clone()),
                    SqlParam::String(user.display_name.clone()),
                    SqlParam::String(user.role.as_str().to_string()),
                    opt_str_param(&user.api_key_hash),
                    SqlParam::I64(user.updated_at),
                    SqlParam::String(user.id.clone()),
                ],
            )
            .await
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM users WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn create_team(&self, team: &Team) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO teams (id, org_id, name, created_at) VALUES (?, ?, ?, ?)",
                &[
                    SqlParam::String(team.id.clone()),
                    SqlParam::String(team.org_id.clone()),
                    SqlParam::String(team.name.clone()),
                    SqlParam::I64(team.created_at),
                ],
            )
            .await
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        self.query_one(
            "SELECT * FROM teams WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_team,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Team {id}")))
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        self.query_all(
            "SELECT * FROM teams WHERE org_id = ?",
            &[SqlParam::String(org_id.to_string())],
            row_to_team,
        )
        .await
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM teams WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO team_members (team_id, user_id, role, joined_at) VALUES (?, ?, ?, ?)",
                &[
                    SqlParam::String(member.team_id.clone()),
                    SqlParam::String(member.user_id.clone()),
                    SqlParam::String(member.role.as_str().to_string()),
                    SqlParam::I64(member.joined_at),
                ],
            )
            .await
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM team_members WHERE team_id = ? AND user_id = ?",
                &[
                    SqlParam::String(team_id.to_string()),
                    SqlParam::String(user_id.to_string()),
                ],
            )
            .await
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        self.query_all(
            "SELECT * FROM team_members WHERE team_id = ?",
            &[SqlParam::String(team_id.to_string())],
            row_to_team_member,
        )
        .await
    }

    async fn create_api_key(&self, key: &ApiKey) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO api_keys (id, user_id, org_id, key_hash, name, scopes_json, expires_at, created_at, revoked_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(key.id.clone()),
                    SqlParam::String(key.user_id.clone()),
                    SqlParam::String(key.org_id.clone()),
                    SqlParam::String(key.key_hash.clone()),
                    SqlParam::String(key.name.clone()),
                    SqlParam::String(key.scopes_json.clone()),
                    opt_i64_param(key.expires_at),
                    SqlParam::I64(key.created_at),
                    opt_i64_param(key.revoked_at),
                ],
            )
            .await
    }

    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        self.query_one(
            "SELECT * FROM api_keys WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_api_key,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("ApiKey {id}")))
    }

    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        self.query_all(
            "SELECT * FROM api_keys WHERE org_id = ?",
            &[SqlParam::String(org_id.to_string())],
            row_to_api_key,
        )
        .await
    }

    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        self.executor
            .execute(
                "UPDATE api_keys SET revoked_at = ? WHERE id = ?",
                &[SqlParam::I64(revoked_at), SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn delete_api_key(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM api_keys WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }
}
