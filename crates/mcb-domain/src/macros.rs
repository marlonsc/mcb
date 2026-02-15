//! Common macros for domain layer
//!
//! All domain macros are centralized here.
//! Macros are ordered by scope: entities → value objects → ports → schema → registry.

// ============================================================================
// Entity Macros
// ============================================================================

/// Implement `BaseEntity` for structs using `EntityMetadata`
#[macro_export]
macro_rules! impl_base_entity {
    ($t:ty) => {
        impl $crate::entities::BaseEntity for $t {
            fn id(&self) -> &str {
                &self.metadata.id
            }
            fn created_at(&self) -> i64 {
                self.metadata.created_at
            }
            fn updated_at(&self) -> i64 {
                self.metadata.updated_at
            }
        }
    };
}

// ============================================================================
// Value Object Macros
// ============================================================================

/// Define a strong-typed UUID identifier for a domain entity.
///
/// Generates a newtype struct wrapping `uuid::Uuid` with full trait implementations
/// including `Display`, `FromStr`, `Serialize`, `Deserialize`, `JsonSchema`, and
/// deterministic v5 UUID derivation via `from_name`.
#[macro_export]
macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            derive_more::Display,
            derive_more::From,
            derive_more::Into,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
        )]
        #[display("{_0}")]
        pub struct $name(uuid::Uuid);

        impl $name {
            /// Generate a new random UUID v4 identifier.
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            /// Wrap an existing [`uuid::Uuid`].
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Derive a deterministic v5 UUID from a human-readable name.
            ///
            /// The namespace is scoped per type so `CollectionId::from_name("x")`
            /// and `SessionId::from_name("x")` produce different UUIDs.
            pub fn from_name(name: &str) -> Self {
                let ns =
                    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, stringify!($name).as_bytes());
                Self(uuid::Uuid::new_v5(&ns, name.as_bytes()))
            }

            /// Parse from any string: tries UUID first, falls back to `from_name`.
            pub fn from_string(s: &str) -> Self {
                match uuid::Uuid::parse_str(s) {
                    Ok(u) => Self(u),
                    Err(_) => Self::from_name(s),
                }
            }

            /// Hyphenated UUID string (allocates).
            pub fn as_str(&self) -> String {
                self.0.to_string()
            }

            /// Consume and return the hyphenated string.
            pub fn into_string(self) -> String {
                self.0.to_string()
            }

            /// Access the inner [`uuid::Uuid`].
            pub fn inner(&self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0.to_string()
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                uuid::Uuid::parse_str(s).map(Self)
            }
        }

        impl AsRef<uuid::Uuid> for $name {
            fn as_ref(&self) -> &uuid::Uuid {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::from_string(s)
            }
        }
    };
}

// ============================================================================
// Port Macros
// ============================================================================

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

// ============================================================================
// Schema DDL Macros
// ============================================================================

/// Define a table schema with less boilerplate
#[macro_export]
macro_rules! table {
    ($name:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::memory::TableDef {
            name: $name.to_string(),
            columns: vec![ $($col),* ],
        }
    };
}

/// Define a column with less boilerplate
#[macro_export]
macro_rules! col {
    ($name:expr, $type:ident) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, pk) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: true,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, unique) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: true,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, nullable) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: false,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, auto) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: true,
            unique: false,
            not_null: true,
            auto_increment: true,
        }
    };
}

/// Define an index with less boilerplate
#[macro_export]
macro_rules! index {
    ($name:expr, $table:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::memory::IndexDef {
            name: $name.to_string(),
            table: $table.to_string(),
            columns: vec![ $($col.to_string()),* ],
        }
    };
}

// ============================================================================
// Registry Macros
// ============================================================================

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
