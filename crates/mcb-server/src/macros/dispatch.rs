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
