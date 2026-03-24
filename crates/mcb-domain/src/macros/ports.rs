//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Port trait definition macros.
//!
//! Used by `ports/` modules for admin interfaces, enum parsing, and metrics.

/// Define an admin service interface trait for a provider type.
///
/// Generates an async trait with `list_providers`, `switch_provider`,
/// `reload_from_config`, and optional extra methods.
#[macro_export]
macro_rules! provider_admin_interface {
    (
        $(#[$meta:meta])*
        trait $trait_name:ident,
        config = $config_ty:ty,
        list_doc = $list_doc:literal,
        extra = { $($extra_methods:tt)* }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        pub trait $trait_name: Send + Sync + std::fmt::Debug {
            #[doc = $list_doc]
            fn list_providers(&self) -> Vec<ProviderInfo>;
            $($extra_methods)*
            /// Switch to a different provider.
            ///
            /// # Errors
            ///
            /// Returns an error if the provider cannot be initialized with the given config.
            fn switch_provider(&self, config: $config_ty) -> std::result::Result<(), String>;
            /// Reload provider from current application config.
            ///
            /// # Errors
            ///
            /// Returns an error if the config is invalid or the provider fails to reinitialize.
            fn reload_from_config(&self) -> std::result::Result<(), String>;
        }
    };
}

/// Define an aggregate trait with automatic blanket implementation.
///
/// Generates a supertrait + blanket `impl<T>` for any `T` that satisfies all components.
///
/// # Example
///
/// ```ignore
/// define_aggregate! {
///     /// Aggregate for org entity management.
///     pub trait OrgEntityRepository = OrgRegistry + UserRegistry + TeamRegistry;
/// }
/// ```
#[macro_export]
macro_rules! define_aggregate {
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident = $first:ident $(+ $rest:ident)*;
    ) => {
        $(#[$meta])*
        $vis trait $name: $first $(+ $rest)* + Send + Sync {}

        impl<T> $name for T where T: $first $(+ $rest)* + Send + Sync {}
    };
}

/// Implement `FromStr` for an enum with case-insensitive string matching
#[macro_export]
macro_rules! impl_from_str {
    ($type:ty, $err_msg:expr, { $($str_val:expr => $variant:expr),* $(,)? }) => {
        impl std::str::FromStr for $type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $( $str_val => Ok($variant), )*
                    _ => Err(format!($err_msg, s)),
                }
            }
        }
    };
}

/// Define a simple CRUD port trait (create, get, list, update, delete).
///
/// Generates an `#[async_trait]` trait with standard CRUD methods.
/// Supports three variants:
///
/// - **Simple** (`get(id)`): Get/delete by `id: &str` only
/// - **Scoped** (`get(scope_id, id)`): Get/delete by `scope_id: &str, id: &str`
/// - **Optional update**: Omit the `update:` line to skip generating `update_*`
///
/// # Example
///
/// ```ignore
/// define_crud_port! {
///     /// Registry for teams.
///     pub trait TeamRegistry {
///         entity: Team,
///         create: create_team,
///         get: get_team(id),
///         list: list_teams(org_id),
///         delete: delete_team(id),
///     }
/// }
///
/// define_crud_port! {
///     /// Registry for plans.
///     pub trait PlanRegistry {
///         entity: Plan,
///         create: create_plan,
///         get: get_plan(org_id, id),
///         list: list_plans(org_id, project_id),
///         update: update_plan,
///         delete: delete_plan(org_id, id),
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_crud_port {
    // ── Scoped variant: get/delete take (scope_id, id) ──
    (
        $(#[$meta:meta])*
        $vis:vis trait $trait_name:ident {
            entity: $entity:ty,
            create: $create_fn:ident,
            get: $get_fn:ident($scope_param:ident, $get_id:ident),
            list: $list_fn:ident($list_p1:ident, $list_p2:ident),
            $(update: $upd_fn:ident,)?
            delete: $del_fn:ident($del_p1:ident, $del_p2:ident) $(,)?
        }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        $vis trait $trait_name: Send + Sync {
            #[doc = concat!("Create a ", stringify!($entity), ".")]
            async fn $create_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            #[doc = concat!("Get a ", stringify!($entity), " by ID.")]
            async fn $get_fn(&self, $scope_param: &str, $get_id: &str) -> $crate::error::Result<$entity>;
            #[doc = concat!("List ", stringify!($entity), " items.")]
            async fn $list_fn(&self, $list_p1: &str, $list_p2: &str) -> $crate::error::Result<Vec<$entity>>;
            $(
                #[doc = concat!("Update a ", stringify!($entity), ".")]
                async fn $upd_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            )?
            #[doc = concat!("Delete a ", stringify!($entity), ".")]
            async fn $del_fn(&self, $del_p1: &str, $del_p2: &str) -> $crate::error::Result<()>;
        }
    };
    // ── Simple variant: get/delete take (id) only ──
    (
        $(#[$meta:meta])*
        $vis:vis trait $trait_name:ident {
            entity: $entity:ty,
            create: $create_fn:ident,
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident($($list_param:ident),+),
            $(update: $upd_fn:ident,)?
            delete: $del_fn:ident($del_id:ident) $(,)?
        }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        $vis trait $trait_name: Send + Sync {
            #[doc = concat!("Create a ", stringify!($entity), ".")]
            async fn $create_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            #[doc = concat!("Get a ", stringify!($entity), " by ID.")]
            async fn $get_fn(&self, $get_id: &str) -> $crate::error::Result<$entity>;
            #[doc = concat!("List ", stringify!($entity), " items.")]
            async fn $list_fn(&self, $($list_param: &str),+) -> $crate::error::Result<Vec<$entity>>;
            $(
                #[doc = concat!("Update a ", stringify!($entity), ".")]
                async fn $upd_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            )?
            #[doc = concat!("Delete a ", stringify!($entity), ".")]
            async fn $del_fn(&self, $del_id: &str) -> $crate::error::Result<()>;
        }
    };
    // ── Unfiltered list variant: list takes no params ──
    (
        $(#[$meta:meta])*
        $vis:vis trait $trait_name:ident {
            entity: $entity:ty,
            create: $create_fn:ident,
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident(),
            $(update: $upd_fn:ident,)?
            delete: $del_fn:ident($del_id:ident) $(,)?
        }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        $vis trait $trait_name: Send + Sync {
            #[doc = concat!("Create a ", stringify!($entity), ".")]
            async fn $create_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            #[doc = concat!("Get a ", stringify!($entity), " by ID.")]
            async fn $get_fn(&self, $get_id: &str) -> $crate::error::Result<$entity>;
            #[doc = concat!("List all ", stringify!($entity), " items.")]
            async fn $list_fn(&self) -> $crate::error::Result<Vec<$entity>>;
            $(
                #[doc = concat!("Update a ", stringify!($entity), ".")]
                async fn $upd_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            )?
            #[doc = concat!("Delete a ", stringify!($entity), ".")]
            async fn $del_fn(&self, $del_id: &str) -> $crate::error::Result<()>;
        }
    };
}

/// Define a read-only port trait (create, get, list — no update/delete).
///
/// # Example
///
/// ```ignore
/// define_readonly_port! {
///     /// Registry for plan versions.
///     pub trait PlanVersionRegistry {
///         entity: PlanVersion,
///         create: create_plan_version,
///         get: get_plan_version(id),
///         list: list_plan_versions_by_plan(plan_id),
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_readonly_port {
    (
        $(#[$meta:meta])*
        $vis:vis trait $trait_name:ident {
            entity: $entity:ty,
            create: $create_fn:ident,
            get: $get_fn:ident($get_id:ident),
            list: $list_fn:ident($($list_param:ident),+) $(,)?
        }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        $vis trait $trait_name: Send + Sync {
            #[doc = concat!("Create a ", stringify!($entity), ".")]
            async fn $create_fn(&self, item: &$entity) -> $crate::error::Result<()>;
            #[doc = concat!("Get a ", stringify!($entity), " by ID.")]
            async fn $get_fn(&self, $get_id: &str) -> $crate::error::Result<$entity>;
            #[doc = concat!("List ", stringify!($entity), " items.")]
            async fn $list_fn(&self, $($list_param: &str),+) -> $crate::error::Result<Vec<$entity>>;
        }
    };
}

/// Create metric labels `HashMap` inline
#[macro_export]
macro_rules! labels {
    () => {
        std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key.to_string(), $value.to_string());)+
        map
    }};
}
