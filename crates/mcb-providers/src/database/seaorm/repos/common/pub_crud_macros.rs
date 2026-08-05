//! Inherent method macros for `SeaORM` repositories.

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
