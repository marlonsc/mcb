//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Server accessor macros.
//!
//! - `impl_arc_accessors!`: Generate `Arc`-cloning accessor methods for struct fields.

/// Generate `Arc`-cloning accessor methods for `McpServer` fields.
macro_rules! impl_arc_accessors {
    ($($(#[doc = $doc:literal])* $name:ident -> $ty:ty => $($path:ident).+),+ $(,)?) => {
        $(
            $(#[doc = $doc])*
            #[must_use]
            pub fn $name(&self) -> Arc<$ty> {
                Arc::clone(&self.$($path).+)
            }
        )+
    };
}
