//! Provider registry infrastructure macros.
//!
//! Used by `registry/` modules for auto-registration via `linkme`.

/// Implement registry infrastructure for a provider type
#[macro_export]
macro_rules! impl_registry {
    (
        provider_trait: $trait:path,
        config_type: $config:ty,
        entry_type: $entry:ident,
        slice_name: $slice:ident,
        resolve_fn: $resolve:ident,
        list_fn: $list:ident
    ) => {
        /// Registry entry for providers
        pub struct $entry {
            /// Unique provider name
            pub name: &'static str,
            /// Human-readable description
            pub description: &'static str,
            /// Factory function to create provider instance
            pub factory: fn(&$config) -> std::result::Result<std::sync::Arc<dyn $trait>, String>,
        }

        #[linkme::distributed_slice]
        /// Stores the static value static value.
        pub static $slice: [$entry] = [..];

        /// Resolve provider by name from registry.
        ///
        /// # Errors
        ///
        /// Returns an error if the requested provider name is not found in the
        /// registry or if the provider factory fails to construct an instance.
        pub fn $resolve(config: &$config) -> $crate::error::Result<std::sync::Arc<dyn $trait>> {
            let provider_name = &config.provider;

            for entry in $slice {
                if entry.name == provider_name {
                    return (entry.factory)(config).map_err(|e| {
                        $crate::error::Error::Configuration {
                            message: e.to_string(),
                            source: None,
                        }
                    });
                }
            }

            let available: Vec<&str> = $slice.iter().map(|e| e.name).collect();

            Err($crate::error::Error::Configuration {
                message: format!(
                    "Unknown provider '{}'. Available providers: {:?}",
                    provider_name, available
                ),
                source: None,
            })
        }

        /// List all registered providers
        pub fn $list() -> Vec<(&'static str, &'static str)> {
            $slice.iter().map(|e| (e.name, e.description)).collect()
        }
    };
}
