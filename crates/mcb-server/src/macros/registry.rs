//! Tool registration macros.
//!
//! Generates schema-factory, dispatch-function, and `linkme` descriptor entries
//! for MCP tools, used by `tools/registry.rs`.

/// Register a tool: generates schema factory, dispatch function, and linkme descriptor.
///
/// Must be invoked in a context where `CallToolRequestParams`, `ToolHandlers`,
/// `ToolCallFuture`, `ToolDescriptor`, `TOOL_DESCRIPTORS`, and `parse_args`
/// are all in scope.
macro_rules! register_tool {
    ($schema_fn:ident, $call_fn:ident, $descriptor:ident, $handler:ident, $args:ty, $name:literal, $desc:literal) => {
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
        // SAFETY: required by linkme::distributed_slice — see safety rationale
        // on TOOL_DESCRIPTORS definition in tools/registry.rs
        #[allow(unsafe_code)]
        #[linkme::distributed_slice(TOOL_DESCRIPTORS)]
        static $descriptor: ToolDescriptor = ToolDescriptor {
            name: $name,
            description: $desc,
            schema: $schema_fn,
            call: $call_fn,
        };
    };
}
