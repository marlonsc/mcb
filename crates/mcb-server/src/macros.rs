//! Common macros for the server layer
//!
//! All server macros are centralized here.
//! Macros are ordered: tool args → entity dispatch → admin → templates → routing.

// ============================================================================
// Tool Argument Macros
// ============================================================================

/// Define a tool argument struct with auto-derives
macro_rules! tool_schema {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($fields:tt)* }) => {
        $(#[$meta])*

        #[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
        $vis struct $name {
            $($fields)*
        }
    };
}

/// Define a tool enum with auto-derives and serde config
macro_rules! tool_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variants:tt)* }) => {
        $(#[$meta])*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "snake_case")]
        $vis enum $name {
            $($variants)*
        }
    };
}

/// Define a standard CRUD action enum (Create, Get, Update, List, Delete) plus extras
macro_rules! tool_crud_action_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($extra:tt)* }) => {
        tool_enum! {
            $(#[$meta])*
            $vis enum $name {
                /// Create a new entity.
                Create,
                /// Get an entity by ID.
                Get,
                /// Update an existing entity.
                Update,
                /// List entities matching criteria.
                List,
                /// Delete an entity by ID.
                Delete,
                $($extra)*
            }
        }
    };
}

/// Define a unified entity args schema with action and resource types
macro_rules! entity_args_schema {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            action: $action_ty:ty,
            action_desc: $action_desc:literal,
            resource: $resource_ty:ty,
            resource_desc: $resource_desc:literal,
            $(
                $(#[$field_meta:meta])*
                $field:ident: $field_ty:ty => $field_desc:literal,
            )*
        }
    ) => {
        tool_schema! {
            $(#[$meta])*
            $vis struct $name {
                #[doc = $action_desc]
                #[schemars(description = $action_desc)]
                pub action: $action_ty,

                #[doc = $resource_desc]
                #[schemars(description = $resource_desc)]
                pub resource: $resource_ty,

                #[doc = "Resource ID (for get/update/delete)"]
                #[schemars(description = "Resource ID (for get/update/delete)")]
                pub id: Option<String>,

                #[doc = "Organization ID (uses default if omitted)"]
                #[schemars(description = "Organization ID (uses default if omitted)")]
                pub org_id: Option<String>,

                $(
                    $(#[$field_meta])*
                    #[schemars(description = $field_desc)]
                    pub $field: $field_ty,
                )*

                #[doc = "Data payload for create/update (JSON object)"]
                #[schemars(
                    description = "Data payload for create/update (JSON object)",
                    with = "serde_json::Value"
                )]
                pub data: Option<serde_json::Value>,
            }
        }
    };
}

// ============================================================================
// Entity Handler Dispatch Macros
// ============================================================================

/// Dispatches `(action, resource)` pairs to handler expressions with an optional fallback.
#[macro_export]
macro_rules! entity_crud_dispatch {
    (
        action = $action:expr,
        resource = $resource:expr,
        fallback = |$unsupported_action:ident, $unsupported_resource:ident| $fallback:expr,
        { $($arms:tt)* }
    ) => {
        match ($action, $resource) {
            $($arms)*
            ($unsupported_action, $unsupported_resource) => $fallback,
        }
    };
    (
        action = $action:expr,
        resource = $resource:expr,
        { $($arms:tt)* }
    ) => {
        match ($action, $resource) {
            $($arms)*
            (action, resource) => Err(rmcp::model::ErrorData::invalid_params(
                format!("Unsupported action {:?} for resource {:?}", action, resource),
                None,
            )),
        }
    };
}

/// Route unified entity args to domain-specific entity handlers.
macro_rules! define_route_method {
    (
        $fn_name:ident,
        $handler_field:ident,
        $args_ty:ty,
        $map_action:ident,
        $map_resource:ident,
        |$args:ident, $action:ident, $resource:ident| $build_args:expr
    ) => {
        async fn $fn_name(&self, args: EntityArgs) -> Result<CallToolResult, McpError> {
            self.route_entity(
                args,
                $map_action,
                $map_resource,
                |$args, $action, $resource| $build_args,
                |tool_args| self.$handler_field.handle(Parameters(tool_args)),
            )
            .await
        }
    };
}

// ============================================================================
// Admin Browse Macros
// ============================================================================

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

// ============================================================================
// Template Macros
// ============================================================================

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
