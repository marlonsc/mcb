//! Rocket route and template macros.
//!
//! Used by `admin/` for browse endpoints and `templates/` for context building.

/// Define a project-scoped REST endpoint for browsing entities
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
        #[get($route)]
        pub async fn $fn_name(
            _auth: AdminAuth,
            state: &State<AdminState>,
            project_id: Option<String>,
        ) -> Result<Json<$response>, (Status, Json<CacheErrorResponse>)> {
            tracing::info!($log_label);
            let items = fetch_project_scoped_entities::<$entity>(
                state,
                $resource,
                project_id,
                $unavailable,
            )
            .await?;

            Ok(build_browse_response(items, |$field, total| $response {
                $field,
                total,
            }))
        }
    };
}

/// Builds an anonymous serializable template context from key/value pairs.
#[macro_export]
macro_rules! context {
    ($($key:ident $(: $value:expr)?),*$(,)?) => {{
        use ::rocket::serde::ser::{Serialize, SerializeMap, Serializer};
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
