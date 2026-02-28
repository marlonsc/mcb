//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Entity handler dispatch macros.
//!
//! Used by `handlers/entities/` for CRUD routing and action mapping.

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
        #[allow(unreachable_patterns)]
        match ($action, $resource) {
            $($arms)*
            _ => Err(rmcp::model::ErrorData::invalid_params(
                format!("Unsupported action {:?} for resource {:?}", $action, $resource),
                None,
            )),
        }
    };
}

/// Generate an action mapping function from `EntityAction` to a domain-specific action enum.
///
/// Two modes:
/// - `allow_all`: maps all 6 actions including `Release`
/// - `reject_release`: maps 5 actions, rejects `Release` with an error
macro_rules! define_action_mapper {
    ($fn_name:ident, $target:ident, allow_all) => {
        fn $fn_name(action: EntityAction) -> Result<$target, McpError> {
            match action {
                EntityAction::Create => Ok($target::Create),
                EntityAction::Get => Ok($target::Get),
                EntityAction::Update => Ok($target::Update),
                EntityAction::List => Ok($target::List),
                EntityAction::Delete => Ok($target::Delete),
                EntityAction::Release => Ok($target::Release),
            }
        }
    };
    ($fn_name:ident, $target:ident, reject_release) => {
        fn $fn_name(action: EntityAction) -> Result<$target, McpError> {
            match action {
                EntityAction::Create => Ok($target::Create),
                EntityAction::Get => Ok($target::Get),
                EntityAction::Update => Ok($target::Update),
                EntityAction::List => Ok($target::List),
                EntityAction::Delete => Ok($target::Delete),
                EntityAction::Release => {
                    Err(unsupported("release action is only valid for assignment"))
                }
            }
        }
    };
}

/// Generate a resource mapping function from `EntityResource` to a domain-specific resource enum.
///
/// Maps named variants 1:1 and returns an error for unrecognized resources.
macro_rules! define_resource_mapper {
    (
        $fn_name:ident, $target:ident, $error_msg:literal,
        { $($source:ident => $dest:ident),+ $(,)? }
    ) => {
        fn $fn_name(resource: EntityResource) -> Result<$target, McpError> {
            match resource {
                $(EntityResource::$source => Ok($target::$dest),)+
                _ => Err(unsupported($error_msg)),
            }
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
