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
            /// Constructor function to create provider instance
            pub build: fn(&$config) -> std::result::Result<std::sync::Arc<dyn $trait>, String>,
        }

        #[linkme::distributed_slice]
        /// Stores the static value static value.
        pub static $slice: [$entry] = [..];

        /// Resolve provider by name from registry.
        ///
        /// # Errors
        ///
        /// Returns an error if the requested provider name is not found in the
        /// registry or if the provider constructor fails to construct an instance.
        pub fn $resolve(config: &$config) -> $crate::error::Result<std::sync::Arc<dyn $trait>> {
            let provider_name = &config.provider;

            for entry in $slice {
                if entry.name == provider_name {
                    return (entry.build)(config).map_err(|e| {
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

/// Generate `new()`, builder methods, and `with_extra()` for a provider config struct.
///
/// Fields marked `into` generate `impl Into<T>` parameters; others take the type directly.
///
/// ```ignore
/// crate::impl_config_builder!(MyConfig {
///     /// Set the model
///     model: with_model(into String),
///     /// Set the dimensions
///     dimensions: with_dimensions(usize),
/// });
/// ```
#[macro_export]
macro_rules! impl_config_builder {
    (
        $config:ident {
            $(
                $(#[doc = $doc:literal])*
                $field:ident : $method:ident ( $($kind:tt)+ )
            ),* $(,)?
        }
    ) => {
        impl $config {
            /// Create a new config with the given provider name
            pub fn new(provider: impl Into<String>) -> Self {
                Self { provider: provider.into(), ..Default::default() }
            }

            $(
                impl_config_builder!(@builder_method $(#[doc = $doc])* ; $field ; $method ; $($kind)+);
            )*

            /// Add extra configuration
            #[must_use]
            pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
                self.extra.insert(key.into(), value.into());
                self
            }
        }
    };

    (@builder_method $(#[$meta:meta])* ; $field:ident ; $method:ident ; into $ty:ty) => {
        $(#[$meta])*
        #[must_use]
        pub fn $method(mut self, value: impl Into<$ty>) -> Self {
            self.$field = Some(value.into());
            self
        }
    };

    (@builder_method $(#[$meta:meta])* ; $field:ident ; $method:ident ; $ty:ty) => {
        $(#[$meta])*
        #[must_use]
        pub fn $method(mut self, value: $ty) -> Self {
            self.$field = Some(value);
            self
        }
    };
}
