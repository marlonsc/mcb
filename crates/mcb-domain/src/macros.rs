//! Common macros for domain layer

/// Implement `FromStr` for an enum with case-insensitive string matching
#[macro_export]
macro_rules! impl_from_str {
    ($type:ty, $err_msg:expr, { $($str_val:expr => $variant:expr),* $(,)? }) => {
        impl std::str::FromStr for $type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $( $str_val => Ok($variant), )*
                    _ => Err(format!($err_msg, s)),
                }
            }
        }
    };
}

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
            pub factory: fn(&$config) -> Result<std::sync::Arc<dyn $trait>, String>,
        }

        #[linkme::distributed_slice]
        pub static $slice: [$entry] = [..];

        /// Resolve provider by name from registry
        pub fn $resolve(config: &$config) -> Result<std::sync::Arc<dyn $trait>, String> {
            let provider_name = &config.provider;

            for entry in $slice {
                if entry.name == provider_name {
                    return (entry.factory)(config);
                }
            }

            let available: Vec<&str> = $slice.iter().map(|e| e.name).collect();

            Err(format!(
                "Unknown provider '{}'. Available providers: {:?}",
                provider_name, available
            ))
        }

        /// List all registered providers
        pub fn $list() -> Vec<(&'static str, &'static str)> {
            $slice.iter().map(|e| (e.name, e.description)).collect()
        }
    };
}
