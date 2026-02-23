//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::{ApiKey, Organization, Team, TeamMember, User};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    ApiKeyRegistry, OrgRegistry, TeamMemberManager, TeamRegistry, UserRegistry,
};
use mcb_domain::ports::{DatabaseExecutor, SqlParam};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};

use crate::database::sqlite::row_convert::FromRow;
use crate::database::sqlite::sea_entities::organization;
use crate::utils::sqlite::query as query_helpers;
use crate::utils::sqlite::row::{opt_i64_param, opt_str_param};

/// SQLite-backed repository for organization, user, team, and API key entities.
///
/// When constructed with [`Self::new_with_sea`], organization CRUD uses SeaORM;
/// otherwise it uses the executor port. Other entities (users, teams, etc.) always use the executor.
pub struct SqliteOrgEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
    sea_conn: Option<sea_orm::DatabaseConnection>,
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

fn org_to_model(org: &Organization) -> organization::ActiveModel {
    organization::ActiveModel {
        id: Set(org.id.clone()),
        name: Set(org.name.clone()),
        slug: Set(org.slug.clone()),
        settings_json: Set(org.settings_json.clone()),
        created_at: Set(org.created_at),
        updated_at: Set(org.updated_at),
        ..Default::default()
    }
}

fn model_to_org(m: organization::Model) -> Organization {
    Organization {
        id: m.id,
        name: m.name,
        slug: m.slug,
        settings_json: m.settings_json,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}

impl SqliteOrgEntityRepository {
    /// Creates a new repository using the provided database executor (no SeaORM).
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self {
            executor,
            sea_conn: None,
        }
    }

    /// Creates a repository with a SeaORM connection for organization CRUD.
    pub fn new_with_sea(
        executor: Arc<dyn DatabaseExecutor>,
        sea_conn: sea_orm::DatabaseConnection,
    ) -> Self {
        Self {
            executor,
            sea_conn: Some(sea_conn),
        }
    }
}

#[async_trait]
/// Persistent organization registry using `SQLite`.
impl OrgRegistry for SqliteOrgEntityRepository {
    /// Creates a new organization.
    async fn create_org(&self, org: &Organization) -> Result<()> {
        if let Some(ref db) = self.sea_conn {
            let am = org_to_model(org);
            am.insert(db)
                .await
                .map_err(|e| Error::memory_with_source("SeaORM insert organization", e))?;
            return Ok(());
        }
        let params = org_insert_params(org);
        query_helpers::execute(&self.executor, INSERT_ORG_SQL, &params).await
    }

    /// Retrieves an organization by ID.
    async fn get_org(&self, id: &str) -> Result<Organization> {
        if let Some(ref db) = self.sea_conn {
            let opt = organization::Entity::find_by_id(id.to_string())
                .one(db)
                .await
                .map_err(|e| Error::memory_with_source("SeaORM find organization", e))?;
            return Error::not_found_or(opt.map(model_to_org), "Organization", id);
        }
        let org = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM organizations WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            Organization::from_row,
        )
        .await?;
        Error::not_found_or(org, "Organization", id)
    }

    /// Lists all organizations.
    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        if let Some(ref db) = self.sea_conn {
            let models = organization::Entity::find()
                .all(db)
                .await
                .map_err(|e| Error::memory_with_source("SeaORM list organizations", e))?;
            return Ok(models.into_iter().map(model_to_org).collect());
        }
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM organizations",
            &[],
            Organization::from_row,
            "org entity",
        )
        .await
    }

    /// Updates an existing organization.
    async fn update_org(&self, org: &Organization) -> Result<()> {
        if let Some(ref db) = self.sea_conn {
            let am = org_to_model(org);
            am.update(db)
                .await
                .map_err(|e| Error::memory_with_source("SeaORM update organization", e))?;
            return Ok(());
        }
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
        if let Some(ref db) = self.sea_conn {
            if let Some(active) = organization::Entity::find_by_id(id.to_string())
                .one(db)
                .await
                .map_err(|e| Error::memory_with_source("SeaORM find for delete", e))?
                .map(organization::ActiveModel::from)
            {
                active
                    .delete(db)
                    .await
                    .map_err(|e| Error::memory_with_source("SeaORM delete organization", e))?;
            }
            return Ok(());
        }
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
        let user = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM users WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            User::from_row,
        )
        .await?;
        Error::not_found_or(user, "User", id)
    }

    /// Retrieves a user by email within an organization.
    async fn get_user_by_email(&self, org_id: &str, email: &str) -> Result<User> {
        let user = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM users WHERE org_id = ? AND email = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(email.to_owned()),
            ],
            User::from_row,
        )
        .await?;
        Error::not_found_or(user, "User", email)
    }

    /// Lists users in an organization.
    async fn list_users(&self, org_id: &str) -> Result<Vec<User>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM users WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            User::from_row,
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
        let team = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM teams WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            Team::from_row,
        )
        .await?;
        Error::not_found_or(team, "Team", id)
    }

    /// Lists teams in an organization.
    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM teams WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            Team::from_row,
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
            TeamMember::from_row,
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
        let key = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM api_keys WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            ApiKey::from_row,
        )
        .await?;
        Error::not_found_or(key, "ApiKey", id)
    }

    /// Lists API keys for an organization.
    async fn list_api_keys(&self, org_id: &str) -> Result<Vec<ApiKey>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM api_keys WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            ApiKey::from_row,
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
