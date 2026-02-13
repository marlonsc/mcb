//! Violation Definition Macro
//!
//! Provides a declarative macro for defining violation enums with
//! automatic trait implementations.
//!
//! # Example
//!
//! ```text
//! define_violations! {
//!     ViolationCategory::Architecture,
//!     pub enum DependencyViolation {
//!         /// Doc comments are preserved
//!         violation(
//!             id = "DEP001",
//!             severity = Error,
//!             message = "Forbidden Cargo dependency: {crate_name} depends on {forbidden_dep}"
//!         )
//!         ForbiddenCargoDependency {
//!             crate_name: String,
//!             forbidden_dep: String,
//!             file: PathBuf,
//!             line: usize,
//!         },
//!     }
//! }
//! ```

/// Macro to define violation enums with automatic trait implementations
///
/// This macro generates:
/// - The enum with all variants
/// - `Display` implementation with formatted messages (optional)
/// - `Violation` trait implementation
///
/// # Parameters
///
/// - `no_display` (optional): If present, `Display` impl is skipped (manual impl required)
/// - `$category`: The `ViolationCategory` for all variants
/// - `$vis`: Visibility modifier (pub, pub(crate), etc.)
/// - `$name`: Name of the enum
/// - For each variant:
///   - `violation(...)` block defining metadata
///   - Variant definition with fields
#[macro_export]
macro_rules! define_violations {
    // Branch: No Display (Manual Display implementation)
    (
        no_display,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[$meta:meta])*
                violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )
                $variant:ident {
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $field_ty:ty
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        define_violations!(
            @generate_core,
            $category,
            $vis enum $name {
                $(
                    { $(#[$meta])* }
                    { $id, $severity, $($msg)?, $($suggestion)? }
                    $variant {
                        $(
                            { $(#[$field_meta])* }
                            $field : $field_ty
                        ),*
                    }
                ),*
            }
        );
    };

    // Branch: Display Enabled (Default)
    (
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[$meta:meta])*
                violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )
                $variant:ident {
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $field_ty:ty
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        define_violations!(
            @generate_core,
            $category,
            $vis enum $name {
                $(
                    { $(#[$meta])* }
                    { $id, $severity, $($msg)?, $($suggestion)? }
                    $variant {
                        $(
                            { $(#[$field_meta])* }
                            $field : $field_ty
                        ),*
                    }
                ),*
            }
        );

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@format f, $($msg,)? $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }
        }
    };

    // Core Generator (Recieves normalized args)
    (
        @generate_core,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                { $(#[$meta:meta])* }
                { $id:literal, $severity:ident, $($msg:literal)?, $($suggestion:literal)? }
                $variant:ident {
                    $(
                        { $(#[$field_meta:meta])* }
                        $field:ident : $field_ty:ty
                    ),*
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        $vis enum $name {
            $(
                $(#[$meta])*
                $variant {
                    $(
                        $(#[$field_meta])*
                        $field: $field_ty
                    ),*
                }
            ),*
        }

        impl $crate::violation_trait::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::violation_trait::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::violation_trait::Severity {
                match self {
                    $( Self::$variant { .. } => $crate::violation_trait::Severity::$severity ),*
                }
            }

            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            #[allow(unused_variables)]
                            let result = {
                                define_violations!(@find_file $( $field : $field_ty ),*)
                            };
                            result
                        }
                    ),*
                }
            }

            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            #[allow(unused_variables)]
                            let result = {
                                define_violations!(@find_line $( $field : $field_ty ),*)
                            };
                            result
                        }
                    ),*
                }
            }

            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { .. } => {
                            $( return Some($suggestion.to_string()); )?
                            #[allow(unreachable_code)]
                            None
                        }
                    ),*
                }
            }
        }
    };

    // Helper to find 'file' or 'path' or 'location' field
    (@find_file $( $field:ident : $field_ty:ty ),*) => {{
        $(
            if let Some(f) = define_violations!(@check_file $field : $field_ty) {
                return Some(f);
            }
        )*
        None
    }};

    (@check_file $field:ident : PathBuf) => { if stringify!($field) == "file" || stringify!($field) == "path" || stringify!($field) == "location" { Some($field) } else { None } };
    (@check_file $field:ident : std::path::PathBuf) => { if stringify!($field) == "file" || stringify!($field) == "path" || stringify!($field) == "location" { Some($field) } else { None } };
    (@check_file $field:ident : $ty:ty) => { None };

    // Helper to find 'line' field
    (@find_line $( $field:ident : $field_ty:ty ),*) => {{
        $(
            if let Some(l) = define_violations!(@check_line $field : $field_ty) {
                return Some(l);
            }
        )*
        None
    }};

    (@check_line $field:ident : usize) => { if stringify!($field) == "line" { Some(*$field) } else { None } };
    (@check_line $field:ident : $ty:ty) => { None };

    // Format helper - with message template
    (@format $f:ident, $msg:literal, $( $field:ident : $ty:ty ),*) => {{
        $(
            #[allow(unused_variables)]
            let $field = define_violations!(@fmt_val $field : $ty);
        )*
        write!($f, $msg)
    }};

    // Format helper - no message template (use Debug)
    (@format $f:ident, $( $field:ident : $ty:ty ),*) => {
        write!($f, "{:?}", ($( $field ),*))
    };

    // Helper to format values for Display
    (@fmt_val $val:ident : PathBuf) => { $val.display() };
    (@fmt_val $val:ident : std::path::PathBuf) => { $val.display() };
    (@fmt_val $val:ident : Option<PathBuf>) => {
        $val.as_ref().map(|p| p.display().to_string()).unwrap_or_default()
    };
    (@fmt_val $val:ident : $ty:ty) => { $val };
}

/// Macro to implement the `Validator` trait for types that have a `validate_all()` method.
#[macro_export]
macro_rules! impl_validator {
    ($validator_ty:ty, $name:literal, $desc:literal) => {
        impl $crate::validator_trait::Validator for $validator_ty {
            fn name(&self) -> &'static str {
                $name
            }

            fn description(&self) -> &'static str {
                $desc
            }

            fn validate(
                &self,
                _config: &$crate::ValidationConfig,
            ) -> anyhow::Result<Vec<Box<dyn $crate::violation_trait::Violation>>> {
                let violations = self.validate_all()?;
                Ok(violations
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn $crate::violation_trait::Violation>)
                    .collect())
            }
        }
    };
}
