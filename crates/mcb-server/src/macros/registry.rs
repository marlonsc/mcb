//! Tool registration macros.
//!
//! Generates schema-factory, dispatch-function, and `linkme` descriptor entries
//! for MCP tools, used by `tools/registry.rs`.

/// Register a tool: generates schema factory, dispatch function, and linkme descriptor.
///
/// Two forms:
/// - **Direct**: args type matches handler signature.
/// - **Mapped**: args convert via `From` before dispatch (`$args => $target`).
///
/// Must be invoked in a context where `CallToolRequestParams`, `ToolHandlers`,
/// `ToolCallFuture`, `ToolDescriptor`, `TOOL_DESCRIPTORS`, and `parse_args`
/// are all in scope.
macro_rules! register_tool {
    // Direct dispatch — args go straight to handler
    ($schema_fn:ident, $call_fn:ident, $descriptor:ident, $handler:ident, $args:ty, $name:literal, $desc:expr) => {
        fn $schema_fn() -> schemars::Schema {
            schemars::schema_for!($args)
        }
        fn $call_fn<'a>(
            request: &'a CallToolRequestParams,
            handlers: &'a ToolHandlers,
        ) -> ToolCallFuture<'a> {
            Box::pin(async move {
                let args = parse_args::<$args>(request)?;
                handlers.$handler.handle(Parameters(args)).await
            })
        }
        #[linkme::distributed_slice(TOOL_DESCRIPTORS)]
        static $descriptor: ToolDescriptor = ToolDescriptor {
            name: $name,
            description: $desc,
            schema: $schema_fn,
            call: $call_fn,
        };
    };
    // Mapped dispatch — args convert via From<A> for B before handler
    ($schema_fn:ident, $call_fn:ident, $descriptor:ident, $handler:ident, $args:ty => $target:ty, $name:literal, $desc:expr) => {
        fn $schema_fn() -> schemars::Schema {
            schemars::schema_for!($args)
        }
        fn $call_fn<'a>(
            request: &'a CallToolRequestParams,
            handlers: &'a ToolHandlers,
        ) -> ToolCallFuture<'a> {
            Box::pin(async move {
                let args = parse_args::<$args>(request)?;
                let mapped: $target = args.into();
                handlers.$handler.handle(Parameters(mapped)).await
            })
        }
        #[linkme::distributed_slice(TOOL_DESCRIPTORS)]
        static $descriptor: ToolDescriptor = ToolDescriptor {
            name: $name,
            description: $desc,
            schema: $schema_fn,
            call: $call_fn,
        };
    };
}
