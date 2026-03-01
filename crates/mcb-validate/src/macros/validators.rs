//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
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
///
/// Generates a `checks()` that wraps `validate_all()` as a single named check.
/// The default `validate()` on the trait handles timing and logging.
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

            fn checks<'a>(
                &'a self,
                _config: &'a $crate::ValidationConfig,
            ) -> $crate::Result<Vec<$crate::traits::validator::NamedCheck<'a>>> {
                Ok(vec![$crate::traits::validator::NamedCheck::new(
                    "validate_all",
                    move || {
                        let violations = self.validate_all()?;
                        Ok(violations
                            .into_iter()
                            .map(|v| Box::new(v) as Box<dyn $crate::traits::violation::Violation>)
                            .collect())
                    },
                )])
            }
        }
    };
}
/// Defines a standard simple validator that executes a list of functions.
///
/// Generates `checks()` with one [`NamedCheck`] per function in the list.
/// The default `validate()` on the trait runs them through `run_checks()`.
/// Also generates a typed `validate_all()` convenience method for direct use.
#[macro_export]
macro_rules! create_validator {
    ($name:ident, $id:literal, $desc:literal, $violation_ty:ty, [ $($func:path),* $(,)? ]) => {
        #[doc = $desc]
        pub struct $name {
            pub(crate) config: $crate::ValidationConfig,
        }

        $crate::impl_simple_validator_new!($name);

        impl $name {
            /// Runs all checks and returns typed violations.
            ///
            /// # Errors
            ///
            /// Returns an error if any sub-check fails.
            pub fn validate_all(&self) -> $crate::Result<Vec<$violation_ty>> {
                let mut violations = Vec::new();
                $(
                    violations.extend($func(&self.config)?);
                )*
                Ok(violations)
            }
        }

        impl $crate::traits::validator::Validator for $name {
            fn name(&self) -> &'static str {
                $id
            }

            fn description(&self) -> &'static str {
                $desc
            }

            fn checks<'a>(
                &'a self,
                _config: &'a $crate::ValidationConfig,
            ) -> $crate::Result<Vec<$crate::traits::validator::NamedCheck<'a>>> {
                Ok(vec![
                    $(
                        $crate::traits::validator::NamedCheck::new(
                            stringify!($func),
                            move || {
                                let violations = $func(&self.config)?;
                                Ok(violations
                                    .into_iter()
                                    .map(|v| {
                                        Box::new(v)
                                            as Box<dyn $crate::traits::violation::Violation>
                                    })
                                    .collect())
                            },
                        ),
                    )*
                ])
            }
        }
    };
}

/// Defines a complete validator including its struct, violation enum, and traits.
///
/// This unified macro replaces the need for separate `define_violations!`,
/// `pub struct`, and `impl Validator` blocks.
///
/// It generates:
/// 1. The violation enum (via `define_violations!`)
/// 2. The validator struct
/// 3. A `validate_all(&self)` convenience method
/// 4. The `Validator` trait implementation
///
/// Note: You still need to implement `new()` and/or `with_config()` yourself
/// in a separate `impl` block, as configuration loading varies between validators.
#[macro_export]
macro_rules! define_validator {
    (
        name: $name_str:literal,
        description: $desc_str:literal,

        $(#[$struct_meta:meta])*
        $struct_vis:vis struct $validator_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }

        violations: $severity_mode:ident, $category:expr,
        $(#[$enum_meta:meta])*
        $enum_vis:vis enum $enum_name:ident {
            $($violation_body:tt)*
        }

        checks: [
            $( $check_name:ident => $check_expr:expr ),* $(,)?
        ]

        $(, enabled = $enabled_expr:expr )?
        $(,)?
    ) => {
        use $crate::define_violations;
        $crate::define_violations! {
            $severity_mode,
            $category,
            $(#[$enum_meta])*
            $enum_vis enum $enum_name {
                $($violation_body)*
            }
        }

        $(#[$struct_meta])*
        $struct_vis struct $validator_name {
            $(
                $(#[$field_meta])*
                $field_vis $field : $field_ty,
            )*
        }

        impl $validator_name {
            /// Runs all checks and returns typed violations.
            ///
            /// # Errors
            ///
            /// Returns an error if any sub-check fails.
            pub fn validate_all(&self) -> $crate::Result<Vec<$enum_name>> {
                $(
                    // If an enabled check is provided, evaluate it
                    if !(($enabled_expr)(self)) {
                        return Ok(Vec::new());
                    }
                )?

                let mut violations = Vec::new();
                $(
                    violations.extend($check_expr(self)?);
                )*
                Ok(violations)
            }
        }

        impl $crate::traits::validator::Validator for $validator_name {
            fn name(&self) -> &'static str {
                $name_str
            }

            fn description(&self) -> &'static str {
                $desc_str
            }

            fn checks<'a>(
                &'a self,
                _config: &'a $crate::ValidationConfig,
            ) -> $crate::Result<Vec<$crate::traits::validator::NamedCheck<'a>>> {
                $(
                    if !(($enabled_expr)(self)) {
                        return Ok(Vec::new());
                    }
                )?
                Ok(vec![
                    $(
                        $crate::traits::validator::NamedCheck::new(
                            stringify!($check_name),
                            move || {
                                let violations = $check_expr(self)?;
                                Ok(violations
                                    .into_iter()
                                    .map(|v| Box::new(v) as Box<dyn $crate::traits::violation::Violation>)
                                    .collect())
                            },
                        ),
                    )*
                ])
            }
        }
    };
}
