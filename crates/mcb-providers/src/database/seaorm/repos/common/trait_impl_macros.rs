//! Trait implementation macros built on the low-level repository macros.

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
