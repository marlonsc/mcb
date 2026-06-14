//! Service resolution macros.
//!
//! Helper macro for generating typed service resolver functions
//! from the distributed-slice service registry.

/// Generate a typed service resolver function.
///
/// Creates a public function that looks up a service name in the
/// `SERVICES_REGISTRY` distributed slice and downcasts the builder
/// to the expected variant.
macro_rules! resolve_service {
    ($fn_name:ident, $const_expr:expr, $variant:ident, $trait_obj:ty) => {
        /// Resolve a service by name from the registry.
        ///
        /// # Errors
        ///
        /// Returns an error if the service provider is not registered or has the wrong variant.
        pub fn $fn_name(context: &dyn std::any::Any) -> Result<std::sync::Arc<$trait_obj>> {
            match find_builder($const_expr)? {
                ServiceBuilder::$variant(build) => build(context),
                _ => Err(Error::internal(format!(
                    "Service provider '{}' is not a {} builder",
                    $const_expr,
                    stringify!($variant)
                ))),
            }
        }
    };
}
