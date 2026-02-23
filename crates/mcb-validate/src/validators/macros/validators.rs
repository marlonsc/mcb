//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Validator registration and trait implementation macros.
//!
//! Used by `validators/` for building registries and implementing the `Validator` trait.

/// Build a `ValidatorRegistry` from validator types that expose `new(&Path)`.
#[macro_export]
macro_rules! mk_validators {
    ($root:expr; $( $validator:path ),+ $(,)?) => {{
        let mut registry = $crate::traits::validator::ValidatorRegistry::new();
        $(
            registry = registry.with_validator(Box::new(<$validator>::new($root)));
        )+
        registry
    }};
}

/// Generates `new(workspace_root)` and `with_config(config)` for simple validators
/// that store only a `config: ValidationConfig` field.
///
/// Use for validators whose `with_config` simply stores the config (Type A).
#[macro_export]
macro_rules! impl_simple_validator_new {
    ($struct_name:ident) => {
        impl $struct_name {
            /// Create a new validator for the given workspace root.
            pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
                Self::with_config($crate::ValidationConfig::new(workspace_root))
            }

            /// Create a validator with custom configuration.
            #[must_use]
            pub fn with_config(config: $crate::ValidationConfig) -> Self {
                Self { config }
            }
        }
    };
}

/// Generates `new(workspace_root)` for validators that load `FileConfig` and pass
/// a rules sub-config to their `with_config` method.
///
/// The validator must already define `with_config(config, &rules)`.
#[macro_export]
macro_rules! impl_rules_validator_new {
    ($struct_name:ident, $rules_field:ident) => {
        impl $struct_name {
            /// Create a new validator, loading configuration from files.
            pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
                let root: std::path::PathBuf = workspace_root.into();
                let file_config = $crate::config::FileConfig::load(&root);
                Self::with_config(
                    $crate::ValidationConfig::new(root),
                    &file_config.rules.$rules_field,
                )
            }
        }
    };
}

/// Generates `new(workspace_root)` for validators that load `FileConfig` and pass
/// only a rules sub-config to `with_config` (no `ValidationConfig`).
///
/// Use for validators that don't store `ValidationConfig` (Type C).
#[macro_export]
macro_rules! impl_config_only_validator_new {
    ($struct_name:ident, $rules_field:ident) => {
        impl $struct_name {
            /// Create a new validator, loading configuration from files.
            pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
                let file_config = $crate::config::FileConfig::load(workspace_root);
                Self::with_config(&file_config.rules.$rules_field)
            }
        }
    };
}

/// Implements the `Validator` trait for validators exposing `validate_all()`.
#[macro_export]
macro_rules! impl_validator {
    ($validator_ty:ty, $name:literal, $desc:literal) => {
        impl $crate::traits::validator::Validator for $validator_ty {
            fn name(&self) -> &'static str {
                $name
            }

            fn description(&self) -> &'static str {
                $desc
            }

            fn validate(
                &self,
                _config: &$crate::ValidationConfig,
            ) -> $crate::Result<Vec<Box<dyn $crate::traits::violation::Violation>>> {
                let violations = self.validate_all()?;
                Ok(violations
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn $crate::traits::violation::Violation>)
                    .collect())
            }
        }
    };
}
/// Defines a standard simple validator that executes a list of functions.
#[macro_export]
macro_rules! create_validator {
    ($name:ident, $id:literal, $desc:literal, $violation_ty:ty, [ $($func:path),* $(,)? ]) => {
        #[doc = $desc]
        pub struct $name {
            pub(crate) config: $crate::ValidationConfig,
        }

        $crate::impl_simple_validator_new!($name);

        impl $name {
            /// Run all validations and return violations of the specific type.
            ///
            /// # Errors
            /// Returns an error if file traversal or pattern compilation fails.
            pub fn validate_all(&self) -> $crate::Result<Vec<$violation_ty>> {
                let mut violations: Vec<$violation_ty> = Vec::new();
                $(
                    violations.extend($func(&self.config)?);
                )*
                Ok(violations)
            }
        }

        $crate::impl_validator!($name, $id, $desc);
    };
}
