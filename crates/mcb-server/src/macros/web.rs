//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Web route and template macros.
//!
//! Used by `admin/` for browse endpoints and `templates/` for context building.

/// Define a project-scoped REST endpoint for browsing entities (Axum).
///
/// Generates an `async fn` handler compatible with [`axum::routing::get`].
/// The handler uses [`AxumAdminAuth`] for authentication and
/// [`axum::extract::Query`] for the optional `project_id` parameter.
///
/// Callers must have `ProjectIdQuery` in scope (defined in `admin::browse`).
macro_rules! define_project_scoped_browse_endpoint {
    (
        $fn_name:ident,
        $route:literal,
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

/// Builds an anonymous serializable template context from key/value pairs.
#[macro_export]
macro_rules! context {
    ($($key:ident $(: $value:expr)?),*$(,)?) => {{
        use ::serde::ser::{Serialize, SerializeMap, Serializer};
        use ::std::fmt::{Debug, Formatter};
        use ::std::result::Result;

        #[allow(non_camel_case_types)]
        struct ContextMacroCtxObject<$($key: Serialize),*> {
            $($key: $key),*
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Serialize),*> Serialize for ContextMacroCtxObject<$($key),*> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut map = serializer.serialize_map(None)?;
                $(map.serialize_entry(stringify!($key), &self.$key)?;)*
                map.end()
            }
        }

        #[allow(non_camel_case_types)]
        impl<$($key: Debug + Serialize),*> Debug for ContextMacroCtxObject<$($key),*> {
            fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct("context!")
                    $(.field(stringify!($key), &self.$key))*
                    .finish()
            }
        }

        ContextMacroCtxObject {
            $($key $(: $value)?),*
        }
    }};
}
