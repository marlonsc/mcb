//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Tool argument definition macros.
//!
//! Used by `args/` modules for MCP tool schema definitions.

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

/// Define a focused MCP tool with minimal visible args, hidden context, and
/// automatic `From<Self> for CompoundArgs` conversion.
///
/// - Visible fields appear in the MCP schema with `#[schemars(description)]`.
/// - Hidden fields get `#[schemars(skip)]` and auto-passthrough in `From`.
/// - Convert maps visible + fixed values to the compound type.
///
/// ```ignore
/// tool_action! {
///     /// Search code by natural language.
///     pub struct SearchCodeArgs => SearchArgs {
///         #[schemars(description = "Query")]
///         #[validate(length(min = 1))]
///         query: String,
///         ;
///         hidden { org_id: Option<String>, repo_id: Option<String> }
///         ;
///         convert |a| { query: a.query, resource: SearchResource::Code }
///     }
/// }
/// ```
macro_rules! tool_action {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident => $compound:ident {
            $(
                $(#[$fmeta:meta])*
                $fname:ident : $fty:ty
            ),* $(,)?
            ;
            hidden { $( $hname:ident : $hty:ty ),* $(,)? }
            ;
            convert |$binding:ident| { $( $cfield:ident : $cexpr:expr ),* $(,)? }
        }
    ) => {
        tool_schema! {
            $(#[$struct_meta])*
            $vis struct $name {
                $(
                    $(#[$fmeta])*
                    #[doc = concat!("See `", stringify!($name), "` tool.")]
                    pub $fname: $fty,
                )*
                $(
                    #[doc = concat!(stringify!($hname), " (auto-injected).")]
                    #[schemars(skip)]
                    pub $hname: $hty,
                )*
            }
        }

        impl From<$name> for $compound {
            fn from($binding: $name) -> Self {
                Self {
                    $( $cfield: $cexpr, )*
                    $( $hname: $binding.$hname, )*
                }
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
                    with = "crate::args::schema_helpers::ObjectDataSchema"
                )]
                pub data: Option<serde_json::Value>,
            }
        }
    };
}
