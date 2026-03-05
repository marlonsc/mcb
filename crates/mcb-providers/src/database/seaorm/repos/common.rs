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

/// Ensures the default organization and a project row exist (idempotent).
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
        slug: Set(DEFAULT_ORG_NAME.to_owned()),
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

// ============================================================================
// Generic CRUD macros for all SeaORM repositories
// ============================================================================
//
// These macros accept a `$db` expression (e.g. `&self.db`, `self.db()`) so
// they work with both `DatabaseConnection` and `Arc<DatabaseConnection>`.

/// Insert a domain entity into the database.
///
/// Converts `$item` via `Into<ActiveModel>`, inserts, and maps errors.
///
/// ```rust,ignore
/// sea_repo_insert!(&self.db, project_phase, phase, "create project phase")
/// ```
macro_rules! sea_repo_insert {
    ($db:expr, $mod:ident, $item:expr, $ctx:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        $mod::Entity::insert(active)
            .exec($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Find a single entity by primary key and return a required result.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_get!(&self.db, project_phase, ProjectPhase, "ProjectPhase", id, "get project phase")
/// ```
macro_rules! sea_repo_get {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $id:expr, $ctx:expr) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

/// Find a single entity by primary key and return an optional result.
///
/// ```rust,ignore
/// sea_repo_get_opt!(&self.db, project_phase, ProjectPhase, id, "get project phase")
/// ```
macro_rules! sea_repo_get_opt {
    ($db:expr, $mod:ident, $type:ty, $id:expr, $ctx:expr) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(model.map(<$type>::from))
    }};
}

/// Find a single entity by primary key with additional column filters.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_get_filtered!(&self.db, project_issue, ProjectIssue, "ProjectIssue", id, "get issue",
///     project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_get_filtered {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $id:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            $(.filter($col.eq($val)))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

/// Update a domain entity by converting to `ActiveModel`.
///
/// ```rust,ignore
/// sea_repo_update!(&self.db, project_phase, phase, "update phase")
/// ```
macro_rules! sea_repo_update {
    ($db:expr, $mod:ident, $item:expr, $ctx:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        active
            .update($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Delete an entity by primary key.
///
/// ```rust,ignore
/// sea_repo_delete!(&self.db, project_phase, id, "delete phase")
/// ```
macro_rules! sea_repo_delete {
    ($db:expr, $mod:ident, $id:expr, $ctx:expr) => {{
        $mod::Entity::delete_by_id($id.to_owned())
            .exec($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Delete an entity by primary key with additional column filters.
///
/// ```rust,ignore
/// sea_repo_delete_filtered!(&self.db, project_issue, id, "delete issue", project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_delete_filtered {
    ($db:expr, $mod:ident, $id:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        use sea_orm::ModelTrait;
        if let Some(m) = $mod::Entity::find_by_id($id.to_owned())
            $(.filter($col.eq($val)))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?
        {
            m.delete($db).await.map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        }
        Ok(())
    }};
}

/// List entities with optional column filters.
///
/// ```rust,ignore
/// sea_repo_list!(&self.db, project_issue, ProjectIssue, "list issues", project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_list {
    ($db:expr, $mod:ident, $type:ty, $ctx:expr $(, $col:expr => $val:expr)*) => {{
        let models = $mod::Entity::find()
            $(.filter($col.eq($val)))*
            .all($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(models.into_iter().map(<$type>::from).collect())
    }};
}

/// Find-by-id, set a single field, and update.
///
/// Handles the common pattern of "find → `not_found` → set field → save".
///
/// ```rust,ignore
/// sea_repo_set_field!(self.db(), api_key, id, "ApiKey", "revoke api key",
///     revoked_at = Some(revoked_at))
/// ```
macro_rules! sea_repo_set_field {
    ($db:expr, $mod:ident, $id:expr, $label:literal, $ctx:expr, $field:ident = $val:expr) => {{
        use sea_orm::ActiveValue;
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        let m = Error::not_found_or(model, $label, $id)?;
        let mut active: $mod::ActiveModel = m.into();
        active.$field = ActiveValue::Set($val);
        active
            .update($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Find a single entity by a specific column and return a required result.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_find_by_column!(&self.db, project, Project, "Project", name,
///     "get project by name", project::Column::OrgId => org_id, project::Column::Name => name)
/// ```
macro_rules! sea_repo_find_by_column {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $key:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        let model = $mod::Entity::find()
            $(.filter($col.eq($val.to_owned())))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $key)
    }};
}

// ============================================================================
// High-level trait implementation macros
// ============================================================================
//
// Generate entire trait implementations from a compact declaration.
// These eliminate the repetitive create/get/list/update/delete boilerplate.
//
// The `db` parameter is the method name on `self` that returns a `&DatabaseConnection`,
// e.g. `db: db` will expand to `self.db()` inside the generated methods.

/// Generate a simple CRUD trait impl (no `org_id` scoping).
///
/// ```rust,ignore
/// sea_impl_crud!(TeamRegistry for SeaOrmEntityRepository { db: db,
///     entity: team, domain: Team, label: "Team",
///     create: create_team(t),
///     get: get_team(id),
///     list: list_teams(team::Column::OrgId => org_id),
///     delete: delete_team(id),
/// });
/// ```
macro_rules! sea_impl_crud {
    // Variant with filtered list
    (
        $trait:ident for $repo:ty { db: $db_method:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident($($list_col:expr => $list_param:ident),+),
            $(update: $upd_fn:ident($upd_p:ident),)?
            delete: $del_fn:ident($del_id:ident)
        }
    ) => {
        #[async_trait]
        impl $trait for $repo {
            async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(self.$db_method(), $mod, $create_p, concat!(stringify!($create_fn)))
            }
            async fn $get_fn(&self, $get_id: &str) -> Result<$dtype> {
                sea_repo_get!(self.$db_method(), $mod, $dtype, $label, $get_id, concat!(stringify!($get_fn)))
            }
            async fn $list_fn(&self, $($list_param: &str),+) -> Result<Vec<$dtype>> {
                sea_repo_list!(self.$db_method(), $mod, $dtype, concat!(stringify!($list_fn)),
                    $($list_col => $list_param),+)
            }
            $(async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(self.$db_method(), $mod, $upd_p, concat!(stringify!($upd_fn)))
            })?
            async fn $del_fn(&self, $del_id: &str) -> Result<()> {
                sea_repo_delete!(self.$db_method(), $mod, $del_id, concat!(stringify!($del_fn)))
            }
        }
    };
    // Variant with unfiltered list (no parameters)
    (
        $trait:ident for $repo:ty { db: $db_method:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident(),
            $(update: $upd_fn:ident($upd_p:ident),)?
            delete: $del_fn:ident($del_id:ident)
        }
    ) => {
        #[async_trait]
        impl $trait for $repo {
            async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(self.$db_method(), $mod, $create_p, concat!(stringify!($create_fn)))
            }
            async fn $get_fn(&self, $get_id: &str) -> Result<$dtype> {
                sea_repo_get!(self.$db_method(), $mod, $dtype, $label, $get_id, concat!(stringify!($get_fn)))
            }
            async fn $list_fn(&self) -> Result<Vec<$dtype>> {
                sea_repo_list!(self.$db_method(), $mod, $dtype, concat!(stringify!($list_fn)))
            }
            $(async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(self.$db_method(), $mod, $upd_p, concat!(stringify!($upd_fn)))
            })?
            async fn $del_fn(&self, $del_id: &str) -> Result<()> {
                sea_repo_delete!(self.$db_method(), $mod, $del_id, concat!(stringify!($del_fn)))
            }
        }
    };
}

/// Generate an org-scoped CRUD trait impl (get/delete filtered by `org_id`).
///
/// ```rust,ignore
/// sea_impl_crud_scoped!(IssueRegistry for SeaOrmEntityRepository { db: db,
///     entity: project_issue, domain: ProjectIssue, label: "Issue",
///     scope_col: project_issue::Column::OrgId,
///     create: create_issue(issue),
///     get: get_issue,
///     list: list_issues(project_issue::Column::ProjectId => project_id),
///     update: update_issue(issue),
///     delete: delete_issue,
/// });
/// ```
macro_rules! sea_impl_crud_scoped {
    (
        $trait:ident for $repo:ty { db: $db_method:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            scope_col: $scope_col:expr,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident,
            list: $list_fn:ident($($list_col:expr => $list_param:ident),+),
            update: $upd_fn:ident($upd_p:ident),
            delete: $del_fn:ident
        }
    ) => {
        #[async_trait]
        impl $trait for $repo {
            async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(self.$db_method(), $mod, $create_p, concat!(stringify!($create_fn)))
            }
            async fn $get_fn(&self, org_id: &str, id: &str) -> Result<$dtype> {
                sea_repo_get_filtered!(self.$db_method(), $mod, $dtype, $label, id,
                    concat!(stringify!($get_fn)), $scope_col => org_id)
            }
            async fn $list_fn(&self, org_id: &str, $($list_param: &str),+) -> Result<Vec<$dtype>> {
                sea_repo_list!(self.$db_method(), $mod, $dtype, concat!(stringify!($list_fn)),
                    $scope_col => org_id, $($list_col => $list_param),+)
            }
            async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(self.$db_method(), $mod, $upd_p, concat!(stringify!($upd_fn)))
            }
            async fn $del_fn(&self, org_id: &str, id: &str) -> Result<()> {
                sea_repo_delete_filtered!(self.$db_method(), $mod, id,
                    concat!(stringify!($del_fn)), $scope_col => org_id)
            }
        }
    };
}

/// Generate a create-get-list only trait impl (no update/delete).
///
/// ```rust,ignore
/// sea_impl_cgl!(PlanVersionRegistry for SeaOrmEntityRepository { db: db,
///     entity: plan_version, domain: PlanVersion, label: "PlanVersion",
///     create: create_plan_version(v),
///     get: get_plan_version(id),
///     list: list_plan_versions_by_plan(plan_version::Column::PlanId => plan_id),
/// });
/// ```
macro_rules! sea_impl_cgl {
    (
        $trait:ident for $repo:ty { db: $db_method:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident($($list_col:expr => $list_param:ident),+)
            $(,)?
        }
    ) => {
        #[async_trait]
        impl $trait for $repo {
            async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(self.$db_method(), $mod, $create_p,
                    concat!(stringify!($create_fn)))
            }
            async fn $get_fn(&self, $get_id: &str) -> Result<$dtype> {
                sea_repo_get!(self.$db_method(), $mod, $dtype, $label, $get_id,
                    concat!(stringify!($get_fn)))
            }
            async fn $list_fn(&self, $($list_param: &str),+) -> Result<Vec<$dtype>> {
                sea_repo_list!(self.$db_method(), $mod, $dtype,
                    concat!(stringify!($list_fn)),
                    $($list_col => $list_param),+)
            }
        }
    };
}

/// Generate a trait impl with org-scoped get/list but simple (id-only) delete.
///
/// ```rust,ignore
/// sea_impl_crud_mixed!(VcsBranchRegistry for SeaOrmEntityRepository { db: db,
///     entity: branch, domain: Branch, label: "Branch",
///     scope_col: branch::Column::OrgId,
///     create: create_branch(b),
///     get: get_branch,
///     list: list_branches(branch::Column::RepositoryId => repository_id),
///     update: update_branch(b),
///     delete: delete_branch(id),
/// });
/// ```
macro_rules! sea_impl_crud_mixed {
    (
        $trait:ident for $repo:ty { db: $db_method:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            scope_col: $scope_col:expr,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident,
            list: $list_fn:ident($($list_col:expr => $list_param:ident),+),
            update: $upd_fn:ident($upd_p:ident),
            delete: $del_fn:ident($del_id:ident)
            $(,)?
        }
    ) => {
        #[async_trait]
        impl $trait for $repo {
            async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(self.$db_method(), $mod, $create_p,
                    concat!(stringify!($create_fn)))
            }
            async fn $get_fn(&self, org_id: &str, id: &str) -> Result<$dtype> {
                sea_repo_get_filtered!(self.$db_method(), $mod, $dtype, $label,
                    id, concat!(stringify!($get_fn)), $scope_col => org_id)
            }
            async fn $list_fn(
                &self, org_id: &str, $($list_param: &str),+
            ) -> Result<Vec<$dtype>> {
                sea_repo_list!(self.$db_method(), $mod, $dtype,
                    concat!(stringify!($list_fn)),
                    $scope_col => org_id, $($list_col => $list_param),+)
            }
            async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(self.$db_method(), $mod, $upd_p,
                    concat!(stringify!($upd_fn)))
            }
            async fn $del_fn(&self, $del_id: &str) -> Result<()> {
                sea_repo_delete!(self.$db_method(), $mod, $del_id,
                    concat!(stringify!($del_fn)))
            }
        }
    };
}

/// Generate `pub` CRUD methods directly on a struct (no trait required).
///
/// Use when the methods are not behind a trait (e.g., phase/decision methods
/// on `SeaOrmProjectRepository`). Eliminates hand-written create/get/list/update/delete.
/// The `list:` and `update:` clauses are optional — omit them if you hand-write
/// those methods (e.g., for custom ordering).
///
/// ```rust,ignore
/// sea_pub_crud!(SeaOrmProjectRepository {
///     db_field: db, entity: project_phase, domain: ProjectPhase, label: "ProjectPhase",
///     create: create_phase(phase),
///     get: get_phase_by_id(id),
///     update: update_phase(phase),
///     delete: delete_phase(id),
/// });
/// ```
macro_rules! sea_pub_crud {
    (
        $repo:ty { db_field: $field:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident($get_id:ident),
            $(list: $list_fn:ident($($list_col:expr => $list_param:ident),+),)?
            $(update: $upd_fn:ident($upd_p:ident),)?
            delete: $del_fn:ident($del_id:ident)
            $(,)?
        }
    ) => {
        impl $repo {
            /// Auto-generated create method.
            ///
            /// # Errors
            /// Returns an error if the database insert fails.
            pub async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(&self.$field, $mod, $create_p, concat!(stringify!($create_fn)))
            }
            /// Auto-generated get-by-id method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the query fails.
            pub async fn $get_fn(&self, $get_id: &str) -> Result<$dtype> {
                sea_repo_get!(&self.$field, $mod, $dtype, $label, $get_id, concat!(stringify!($get_fn)))
            }
            $(/// Auto-generated list method.
            ///
            /// # Errors
            /// Returns an error if the database query fails.
            pub async fn $list_fn(&self, $($list_param: &str),+) -> Result<Vec<$dtype>> {
                sea_repo_list!(&self.$field, $mod, $dtype, concat!(stringify!($list_fn)),
                    $($list_col => $list_param),+)
            })?
            $(/// Auto-generated update method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the update fails.
            pub async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(&self.$field, $mod, $upd_p, concat!(stringify!($upd_fn)))
            })?
            /// Auto-generated delete method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the delete fails.
            pub async fn $del_fn(&self, $del_id: &str) -> Result<()> {
                sea_repo_delete!(&self.$field, $mod, $del_id, concat!(stringify!($del_fn)))
            }
        }
    };
}

/// Generate `pub` org-scoped CRUD methods (get/delete filtered by `org_id`, no trait).
///
/// ```rust,ignore
/// sea_pub_crud_scoped!(SeaOrmProjectRepository {
///     db_field: db, entity: project_issue, domain: ProjectIssue, label: "ProjectIssue",
///     scope_col: project_issue::Column::OrgId,
///     create: create_issue(issue),
///     get: get_issue_by_id,
///     list: list_issues(project_issue::Column::ProjectId => project_id),
///     update: update_issue(issue),
///     delete: delete_issue,
/// });
/// ```
macro_rules! sea_pub_crud_scoped {
    (
        $repo:ty { db_field: $field:ident,
            entity: $mod:ident, domain: $dtype:ty, label: $label:literal,
            scope_col: $scope_col:expr,
            create: $create_fn:ident($create_p:ident),
            get: $get_fn:ident,
            list: $list_fn:ident($($list_col:expr => $list_param:ident),+),
            update: $upd_fn:ident($upd_p:ident),
            delete: $del_fn:ident
            $(,)?
        }
    ) => {
        impl $repo {
            /// Auto-generated create method.
            ///
            /// # Errors
            /// Returns an error if the database insert fails.
            pub async fn $create_fn(&self, $create_p: &$dtype) -> Result<()> {
                sea_repo_insert!(&self.$field, $mod, $create_p, concat!(stringify!($create_fn)))
            }
            /// Auto-generated scoped get method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the query fails.
            pub async fn $get_fn(&self, org_id: &str, id: &str) -> Result<$dtype> {
                sea_repo_get_filtered!(&self.$field, $mod, $dtype, $label, id,
                    concat!(stringify!($get_fn)), $scope_col => org_id)
            }
            /// Auto-generated scoped list method.
            ///
            /// # Errors
            /// Returns an error if the database query fails.
            pub async fn $list_fn(&self, org_id: &str, $($list_param: &str),+) -> Result<Vec<$dtype>> {
                sea_repo_list!(&self.$field, $mod, $dtype, concat!(stringify!($list_fn)),
                    $scope_col => org_id, $($list_col => $list_param),+)
            }
            /// Auto-generated update method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the update fails.
            pub async fn $upd_fn(&self, $upd_p: &$dtype) -> Result<()> {
                sea_repo_update!(&self.$field, $mod, $upd_p, concat!(stringify!($upd_fn)))
            }
            /// Auto-generated scoped delete method.
            ///
            /// # Errors
            /// Returns an error if the entity is not found or the delete fails.
            pub async fn $del_fn(&self, org_id: &str, id: &str) -> Result<()> {
                sea_repo_delete_filtered!(&self.$field, $mod, id,
                    concat!(stringify!($del_fn)), $scope_col => org_id)
            }
        }
    };
}
