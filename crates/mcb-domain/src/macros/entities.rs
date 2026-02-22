//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Entity and value-object macros.
//!
//! Used by `entities/` and `value_objects/` modules.

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

/// Define an entity with a selected set of shared fields.
#[macro_export]
macro_rules! define_entity {
    ($name:ident { $($field:ident),* $(,)? }) => {
        define_entity! { pub struct $name { $($field),* } {} }
    };
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($field:ident),* $(,)? } { $($body:tt)* }) => {
        define_entity!(
            @collect
            [$(#[$meta])*]
            [$vis]
            [$name]
            [$($body)*]
            []
            $($field),*
        );
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*]) => {
        $($meta)*
        $vis struct $name {
            $($body)*
            $($shared)*
        }
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*] id $(, $rest:ident)*) => {
        define_entity!(
            @collect
            [$($meta)*]
            [$vis]
            [$name]
            [$($body)*]
            [
                $($shared)*
                /// Unique identifier for the entity.
                pub id: String,
            ]
            $($rest),*
        );
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*] org_id $(, $rest:ident)*) => {
        define_entity!(
            @collect
            [$($meta)*]
            [$vis]
            [$name]
            [$($body)*]
            [
                $($shared)*
                /// Organization identifier for tenant isolation.
                pub org_id: String,
            ]
            $($rest),*
        );
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*] project_id $(, $rest:ident)*) => {
        define_entity!(
            @collect
            [$($meta)*]
            [$vis]
            [$name]
            [$($body)*]
            [
                $($shared)*
                /// Project identifier the entity belongs to.
                pub project_id: String,
            ]
            $($rest),*
        );
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*] created_at $(, $rest:ident)*) => {
        define_entity!(
            @collect
            [$($meta)*]
            [$vis]
            [$name]
            [$($body)*]
            [
                $($shared)*
                /// Timestamp when the entity was created (Unix epoch).
                pub created_at: i64,
            ]
            $($rest),*
        );
    };
    (@collect [$($meta:tt)*] [$vis:vis] [$name:ident] [$($body:tt)*] [$($shared:tt)*] updated_at $(, $rest:ident)*) => {
        define_entity!(
            @collect
            [$($meta)*]
            [$vis]
            [$name]
            [$($body)*]
            [
                $($shared)*
                /// Timestamp when the entity was last updated (Unix epoch).
                pub updated_at: i64,
            ]
            $($rest),*
        );
    };
}

/// Implement `as_str()` by delegating to `AsRefStr::as_ref()`.
#[macro_export]
macro_rules! impl_as_str_from_as_ref {
    ($type:ty) => {
        impl $type {
            /// Returns the canonical string representation.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.as_ref()
            }
        }
    };
}
