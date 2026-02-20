//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::{
    ApiKey, Organization, Team, TeamMember, TeamMemberRole, User, UserRole,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    ApiKeyRegistry, OrgRegistry, TeamMemberManager, TeamRegistry, UserRegistry,
};
use mcb_domain::ports::{DatabaseExecutor, SqlParam, SqlRow};

use crate::utils::sqlite::query as query_helpers;
use crate::utils::sqlite::row::{opt_i64, opt_i64_param, opt_str, opt_str_param, req_i64, req_str};

/// SQLite-backed repository for organization, user, team, and API key entities.
pub struct SqliteOrgEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

const INSERT_ORG_SQL: &str = "INSERT INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)";

fn org_insert_params(org: &Organization) -> [SqlParam; 6] {
    [
        SqlParam::String(org.id.clone()),
        SqlParam::String(org.name.clone()),
        SqlParam::String(org.slug.clone()),
        SqlParam::String(org.settings_json.clone()),
        SqlParam::I64(org.created_at),
        SqlParam::I64(org.updated_at),
    ]
}

impl SqliteOrgEntityRepository {
    /// Creates a new repository using the provided database executor.
    // TODO(qlty): Found 31 lines of similar code in 3 locations (mass = 216)
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

/// Converts a SQL row to an Organization.
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

/// Converts a SQL row to a User.
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

/// Converts a SQL row to a Team.
fn row_to_team(row: &dyn SqlRow) -> Result<Team> {
    Ok(Team {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        name: req_str(row, "name")?,
        created_at: req_i64(row, "created_at")?,
    })
}

use mcb_domain::utils::id;
use mcb_domain::value_objects::ids::TeamMemberId;

// ...

/// Converts a SQL row to a `TeamMember`.
fn row_to_team_member(row: &dyn SqlRow) -> Result<TeamMember> {
    let team_id = req_str(row, "team_id")?;
    let user_id = req_str(row, "user_id")?;
    let id_uuid = id::deterministic("team_member", &format!("{team_id}:{user_id}"));

    Ok(TeamMember {
        id: TeamMemberId::from_uuid(id_uuid),
        team_id,
        user_id,
        role: req_str(row, "role")?
            .parse::<TeamMemberRole>()
            .map_err(|e| Error::memory(format!("Invalid team member role: {e}")))?,
        joined_at: req_i64(row, "joined_at")?,
    })
}

/// Converts a SQL row to an `ApiKey`.
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

#[async_trait]
/// Persistent organization registry using `SQLite`.
impl OrgRegistry for SqliteOrgEntityRepository {
    /// Creates a new organization.
    // TODO(qlty): Found 15 lines of similar code in 2 locations (mass = 91)
    async fn create_org(&self, org: &Organization) -> Result<()> {
        let params = org_insert_params(org);
        query_helpers::execute(&self.executor, INSERT_ORG_SQL, &params).await
    }

    /// Retrieves an organization by ID.
    async fn get_org(&self, id: &str) -> Result<Organization> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM organizations WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_org,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Organization {id}")))
    }

    /// Lists all organizations.
    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM organizations",
            &[],
            row_to_org,
            "org entity",
        )
        .await
    }

    /// Updates an existing organization.
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

    /// Deletes an organization.
    async fn delete_org(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM organizations WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}

#[async_trait]
/// Persistent user registry using `SQLite`.
impl UserRegistry for SqliteOrgEntityRepository {
    /// Creates a new user.
    async fn create_user(&self, user: &User) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO users (id, org_id, email, display_name, role, api_key_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(user.id.clone()),
                    SqlParam::String(user.org_id.clone()),
                    SqlParam::String(user.email.clone()),
                    SqlParam::String(user.display_name.clone()),
                    SqlParam::String(user.role.as_str().to_owned()),
                    opt_str_param(&user.api_key_hash),
                    SqlParam::I64(user.created_at),
                    SqlParam::I64(user.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a user by ID.
    async fn get_user(&self, id: &str) -> Result<User> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM users WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_user,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("User {id}")))
    }

    /// Retrieves a user by email within an organization.
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM users WHERE org_id = ? AND email = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(email.to_owned()),
            ],
            row_to_user,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("User {email}")))
    }

    /// Lists users in an organization.
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM users WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            row_to_user,
            "org entity",
        )
        .await
    }

    /// Updates an existing user.
    async fn update_user(&self, user: &User) -> Result<()> {
        self.executor
            .execute(
                "UPDATE users SET org_id = ?, email = ?, display_name = ?, role = ?, api_key_hash = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(user.org_id.clone()),
                    SqlParam::String(user.email.clone()),
                    SqlParam::String(user.display_name.clone()),
                    SqlParam::String(user.role.as_str().to_owned()),
                    opt_str_param(&user.api_key_hash),
                    SqlParam::I64(user.updated_at),
                    SqlParam::String(user.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a user.
    async fn delete_user(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM users WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}

#[async_trait]
/// Persistent team registry using `SQLite`.
impl TeamRegistry for SqliteOrgEntityRepository {
    /// Creates a new team.
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

    /// Retrieves a team by ID.
    async fn get_team(&self, id: &str) -> Result<Team> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM teams WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_team,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Team {id}")))
    }

    /// Lists teams in an organization.
    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM teams WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            row_to_team,
            "org entity",
        )
        .await
    }

    /// Deletes a team.
    async fn delete_team(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM teams WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}

#[async_trait]
/// Persistent team member manager using `SQLite`.
impl TeamMemberManager for SqliteOrgEntityRepository {
    /// Adds a member to a team.
    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO team_members (team_id, user_id, role, joined_at) VALUES (?, ?, ?, ?)",
                &[
                    SqlParam::String(member.team_id.clone()),
                    SqlParam::String(member.user_id.clone()),
                    SqlParam::String(member.role.as_str().to_owned()),
                    SqlParam::I64(member.joined_at),
                ],
            )
            .await
    }

    /// Removes a member from a team.
    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM team_members WHERE team_id = ? AND user_id = ?",
                &[
                    SqlParam::String(team_id.to_owned()),
                    SqlParam::String(user_id.to_owned()),
                ],
            )
            .await
    }

    /// Lists members of a team.
    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM team_members WHERE team_id = ?",
            &[SqlParam::String(team_id.to_owned())],
            row_to_team_member,
            "org entity",
        )
        .await
    }
}

#[async_trait]
/// Persistent API key registry using `SQLite`.
impl ApiKeyRegistry for SqliteOrgEntityRepository {
    /// Creates a new API key.
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

    /// Retrieves an API key by ID.
    async fn get_api_key(&self, id: &str) -> Result<ApiKey> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM api_keys WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_api_key,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("ApiKey {id}")))
    }

    /// Lists API keys for an organization.
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM api_keys WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            row_to_api_key,
            "org entity",
        )
        .await
    }

    /// Revokes an API key.
    async fn revoke_api_key(&self, id: &str, revoked_at: i64) -> Result<()> {
        self.executor
            .execute(
                "UPDATE api_keys SET revoked_at = ? WHERE id = ?",
                &[SqlParam::I64(revoked_at), SqlParam::String(id.to_owned())],
            )
            .await
    }

    /// Deletes an API key.
    async fn delete_api_key(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM api_keys WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}
