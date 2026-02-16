/// Register a `CacheProvider` with `linkme`.
///
/// This macro generates the factory function header and linkme static registration.
/// The body of the factory function must be provided as a block.
///
/// ## Example
///
/// ```ignore
/// register_cache_provider!(
///     my_provider_factory,
///     config,
///     MY_PROVIDER_STATIC,
///     "my_provider",
///     "My Provider Description",
///     {
///         // factory function body
///         let uri = config.uri.clone().unwrap_or_default();
///         Ok(Arc::new(MyProvider::new(uri)))
///     }
/// );
/// ```
#[macro_export]
macro_rules! register_cache_provider {
    (
        $factory_fn:ident,
        $config_var:ident,
        $static_name:ident,
        $provider_slug:literal,
        $description:literal,
        $body:block
    ) => {
        /// Factory function for creating provider instances.
        fn $factory_fn(
            $config_var: &mcb_domain::registry::cache::CacheProviderConfig,
        ) -> std::result::Result<
            std::sync::Arc<dyn mcb_domain::ports::providers::cache::CacheProvider>,
            String,
        > {
            $body
        }

        #[linkme::distributed_slice(mcb_domain::registry::cache::CACHE_PROVIDERS)]
        static $static_name: mcb_domain::registry::cache::CacheProviderEntry =
            mcb_domain::registry::cache::CacheProviderEntry {
                name: $provider_slug,
                description: $description,
                factory: $factory_fn,
            };
    };
}
