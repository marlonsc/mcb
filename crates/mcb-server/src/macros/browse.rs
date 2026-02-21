//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Browse endpoint macros for the admin API.
//!
//! Generates Axum-compatible handlers for project-scoped entity listing.
//! Used by `admin/browse.rs` to reduce boilerplate across browse endpoints.

/// Define a project-scoped REST endpoint for browsing entities (Axum).
///
/// Generates an `async fn` handler compatible with [`axum::routing::get`].
/// The handler uses [`AxumAdminAuth`](crate::admin::auth::AxumAdminAuth) for
/// authentication and [`axum::extract::Query`] for the optional `project_id`
/// parameter.
///
/// # Parameters
///
/// | Name           | Kind   | Description                                      |
/// |----------------|--------|--------------------------------------------------|
/// | `$fn_name`     | ident  | Name of the generated handler function            |
/// | `$entity`      | path   | Domain entity type (must impl `DeserializeOwned`) |
/// | `$response`    | ident  | Response struct (must have `$field` + `total`)    |
/// | `$field`       | ident  | Field name on the response struct for the items   |
/// | `$resource`    | literal| Entity resource name passed to the backend        |
/// | `$unavailable` | literal| Error message when the backend is unavailable     |
/// | `$log_label`   | literal| `tracing::info!` message emitted on each call     |
/// | `$doc`         | literal| Doc comment attached to the generated function    |
///
/// # Requirements
///
/// Callers must have the following in scope:
/// - `ProjectIdQuery` (defined in `admin::browse`)
/// - `AdminState` (defined in `admin::handlers`)
/// - `CacheErrorResponse` (defined in `admin::handlers`)
/// - `fetch_project_scoped_entities_axum` (defined in `admin::browse`)
/// - `build_browse_response_axum` (defined in `admin::browse`)
///
/// # Example
///
/// ```ignore
/// define_project_scoped_browse_endpoint!(
///     list_browse_repositories,
///     mcb_domain::entities::repository::Repository,
///     RepositoriesBrowseResponse,
///     repositories,
///     "repository",
///     "VCS entity service not available",
///     "list_browse_repositories called",
///     "List repositories for browse entity graph."
/// );
/// ```
macro_rules! define_project_scoped_browse_endpoint {
    (
        $fn_name:ident,
        $entity:path,
        $response:ident,
        $field:ident,
        $resource:literal,
        $unavailable:literal,
        $log_label:literal,
        $doc:literal
    ) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = "Returns `503 Service Unavailable` when backend services are unavailable."]
        pub async fn $fn_name(
            _auth: crate::admin::auth::AxumAdminAuth,
            axum::extract::State(state): axum::extract::State<std::sync::Arc<AdminState>>,
            axum::extract::Query(params): axum::extract::Query<ProjectIdQuery>,
        ) -> Result<axum::Json<$response>, (axum::http::StatusCode, axum::Json<CacheErrorResponse>)>
        {
            tracing::info!($log_label);
            let items = fetch_project_scoped_entities_axum::<$entity>(
                &state,
                $resource,
                params.project_id,
                $unavailable,
            )
            .await?;

            Ok(build_browse_response_axum(items, |$field, total| {
                $response { $field, total }
            }))
        }
    };
}
