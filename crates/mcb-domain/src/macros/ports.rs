//! Port trait definition macros.
//!
//! Used by `ports/` modules for admin interfaces, enum parsing, and metrics.

/// Define an admin service interface trait for a provider type.
///
/// Generates an async trait with `list_providers`, `switch_provider`,
/// `reload_from_config`, and optional extra methods.
#[macro_export]
macro_rules! provider_admin_interface {
    (
        $(#[$meta:meta])*
        trait $trait_name:ident,
        config = $config_ty:ty,
        list_doc = $list_doc:literal,
        extra = { $($extra_methods:tt)* }
    ) => {
        $(#[$meta])*
        #[async_trait::async_trait]
        pub trait $trait_name: Send + Sync + std::fmt::Debug {
            #[doc = $list_doc]
            fn list_providers(&self) -> Vec<ProviderInfo>;
            $($extra_methods)*
            /// Switch to a different provider.
            ///
            /// # Errors
            ///
            /// Returns an error if the provider cannot be initialized with the given config.
            fn switch_provider(&self, config: $config_ty) -> Result<(), String>;
            /// Reload provider from current application config.
            ///
            /// # Errors
            ///
            /// Returns an error if the config is invalid or the provider fails to reinitialize.
            fn reload_from_config(&self) -> Result<(), String>;
        }
    };
}

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

/// Create metric labels `HashMap` inline
#[macro_export]
macro_rules! labels {
    () => {
        std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key.to_string(), $value.to_string());)+
        map
    }};
}
