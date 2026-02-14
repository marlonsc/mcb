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
//!         #[violation(
//!             id = "DEP001",
//!             severity = Error,
//!             message = "Forbidden Cargo dependency: {crate_name} depends on {forbidden_dep}"
//!         )]
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
/// - `Display` implementation with formatted messages
/// - `Violation` trait implementation
///
/// # Parameters
///
/// - `$category`: The `ViolationCategory` for all variants
/// - `$vis`: Visibility modifier (pub, pub(crate), etc.)
/// - `$name`: Name of the enum
/// - For each variant:
///   - `id`: Unique violation identifier (e.g., "DEP001")
///   - `severity`: Error, Warning, or Info (used as default or fallback)
///   - `message`: Display message (can use {`field_name`} placeholders)
///   - `suggestion` (optional): Suggested fix
///   - Fields must include `file: PathBuf` and `line: usize` for location tracking
#[macro_export]
macro_rules! define_violations {
    (
        no_display,
        dynamic_severity,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident {
                    $( $field:ident : $field_ty:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, serde::Serialize)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $variant { $( $field: $field_ty ),* }
            ),*
        }

        impl $crate::traits::violation::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::traits::violation::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::traits::violation::Severity {
                match self {
                    $( Self::$variant { severity, .. } => *severity ),*
                }
            }

            #[allow(unused_variables)]
            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_file $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_line $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@suggestion $($suggestion,)? $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }
        }
    };

    (
        dynamic_severity,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident {
                    $( $field:ident : $field_ty:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, serde::Serialize)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $variant { $( $field: $field_ty ),* }
            ),*
        }

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

        impl $crate::traits::violation::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::traits::violation::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::traits::violation::Severity {
                match self {
                    $( Self::$variant { severity, .. } => *severity ),*
                }
            }

            #[allow(unused_variables)]
            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_file $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_line $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@suggestion $($suggestion,)? $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }
        }
    };

    (
        no_display,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident {
                    $( $field:ident : $field_ty:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, serde::Serialize)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $variant { $( $field: $field_ty ),* }
            ),*
        }

        impl $crate::traits::violation::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::traits::violation::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::traits::violation::Severity {
                match self {
                    $( Self::$variant { .. } => $crate::traits::violation::Severity::$severity ),*
                }
            }

            #[allow(unused_variables)]
            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_file $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_line $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@suggestion $($suggestion,)? $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }
        }
    };

    (
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, message = $msg:literal)?
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident {
                    $( $field:ident : $field_ty:ty ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, serde::Serialize)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $variant { $( $field: $field_ty ),* }
            ),*
        }

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

        impl $crate::traits::violation::Violation for $name {
            fn id(&self) -> &str {
                match self {
                    $( Self::$variant { .. } => $id ),*
                }
            }

            fn category(&self) -> $crate::traits::violation::ViolationCategory {
                $category
            }

            fn severity(&self) -> $crate::traits::violation::Severity {
                match self {
                    $( Self::$variant { .. } => $crate::traits::violation::Severity::$severity ),*
                }
            }

            #[allow(unused_variables)]
            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_file $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn line(&self) -> Option<usize> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@get_line $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }

            #[allow(unused_variables)]
            fn suggestion(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            define_violations!(@suggestion $($suggestion,)? $( $field : $field_ty ),*)
                        }
                    ),*
                }
            }
        }
    };

    // Format helper - with message template
    (@format $f:ident, $msg:literal, $( $field:ident : $field_ty:ty ),*) => {
        write!($f, "{}", define_violations!(@render_template $msg, $( $field : $field_ty ),*))
    };

    // Format helper - no message template (use Debug)
    (@format $f:ident, $( $field:ident : $field_ty:ty ),*) => {
        write!($f, "{:?}", ($( $field ),*))
    };

    // Get file field helper
    (@get_file $( $field:ident : $field_ty:ty ),*) => {{
        #[allow(unused_assignments)]
        #[allow(unused_mut)]
        let mut file_path = None;
        $(
            define_violations!(@check_file_field file_path, $field, $field : $field_ty);
        )*
        file_path
    }};

    (@check_file_field $var:ident, file, $f:ident : $field_ty:ty) => { $var = Some($f) };
    (@check_file_field $var:ident, location, $f:ident : $field_ty:ty) => { $var = Some($f) };
    (@check_file_field $var:ident, path, $f:ident : $field_ty:ty) => { $var = Some($f) };
    (@check_file_field $var:ident, source_file, $f:ident : $field_ty:ty) => { $var = Some($f) };
    (@check_file_field $var:ident, referencing_file, $f:ident : $field_ty:ty) => { $var = Some($f) };
    (@check_file_field $var:ident, locations, $f:ident : Vec<PathBuf>) => { $var = $f.first() };
    (@check_file_field $var:ident, locations, $f:ident : Vec<std::path::PathBuf>) => { $var = $f.first() };
    (@check_file_field $var:ident, $ignore:ident, $f:ident : $field_ty:ty) => {};

    // Get line field helper
    (@get_line $( $field:ident : $field_ty:ty ),*) => {{
        #[allow(unused_assignments)]
        #[allow(unused_mut)]
        let mut line_num = None;
        $(
            define_violations!(@check_line_field line_num, $field, $field : $field_ty);
        )*
        line_num
    }};

    (@check_line_field $var:ident, line, $f:ident : $field_ty:ty) => { $var = Some(*$f) };
    (@check_line_field $var:ident, $ignore:ident, $f:ident : $field_ty:ty) => {};

    // Suggestion helper - with suggestion template
    (@suggestion $suggestion:literal, $( $field:ident : $field_ty:ty ),*) => {
        Some(define_violations!(@render_template $suggestion, $( $field : $field_ty ),*))
    };

    (@suggestion $( $field:ident : $field_ty:ty ),*) => {
        define_violations!(@find_suggestion_field $( $field : $field_ty ),*)
    };

    (@find_suggestion_field suggestion : String, $( $rest:ident : $rest_ty:ty ),*) => {
        Some(suggestion.clone())
    };
    (@find_suggestion_field suggestion : String) => {
        Some(suggestion.clone())
    };
    (@find_suggestion_field $other:ident : $other_ty:ty, $( $rest:ident : $rest_ty:ty ),*) => {
        define_violations!(@find_suggestion_field $( $rest : $rest_ty ),*)
    };
    (@find_suggestion_field $other:ident : $other_ty:ty) => {
        None
    };
    (@find_suggestion_field) => {
        None
    };

    (@render_template $template:literal, $( $field:ident : $field_ty:ty ),*) => {{
        let mut rendered = $template.to_string();
        $(
            let formatted = $crate::violation_macro::violation_field_fmt::ViolationFieldFmt::fmt_field($field);
            rendered = rendered.replace(concat!("{", stringify!($field), "}"), &formatted);
        )*
        rendered
    }};
}

/// Trait-based field formatting for violation message templates.
///
/// Macro `@field_to_string` arms cannot match opaque `:ty` fragments,
/// so we dispatch via trait impls instead. Each supported type gets a
/// specialized impl that renders the field without Debug quoting.
pub mod violation_field_fmt {
    use std::path::PathBuf;

    /// Trait for formatting fields in violation messages.
    pub trait ViolationFieldFmt {
        /// Format the field value for display in a violation message.
        fn fmt_field(&self) -> String;
    }

    impl ViolationFieldFmt for PathBuf {
        fn fmt_field(&self) -> String {
            format!("{}", self.display())
        }
    }

    impl ViolationFieldFmt for String {
        fn fmt_field(&self) -> String {
            self.clone()
        }
    }

    impl ViolationFieldFmt for &str {
        fn fmt_field(&self) -> String {
            (*self).to_string()
        }
    }

    impl ViolationFieldFmt for usize {
        fn fmt_field(&self) -> String {
            self.to_string()
        }
    }

    impl ViolationFieldFmt for u32 {
        fn fmt_field(&self) -> String {
            self.to_string()
        }
    }

    impl ViolationFieldFmt for i32 {
        fn fmt_field(&self) -> String {
            self.to_string()
        }
    }

    impl ViolationFieldFmt for i64 {
        fn fmt_field(&self) -> String {
            self.to_string()
        }
    }

    impl ViolationFieldFmt for bool {
        fn fmt_field(&self) -> String {
            self.to_string()
        }
    }

    impl ViolationFieldFmt for Vec<String> {
        fn fmt_field(&self) -> String {
            self.join(", ")
        }
    }

    impl ViolationFieldFmt for Vec<PathBuf> {
        fn fmt_field(&self) -> String {
            self.iter()
                .map(|p| format!("{}", p.display()))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    impl ViolationFieldFmt for crate::Severity {
        fn fmt_field(&self) -> String {
            format!("{self}")
        }
    }

    impl ViolationFieldFmt for crate::ComponentType {
        fn fmt_field(&self) -> String {
            format!("{self}")
        }
    }

    impl ViolationFieldFmt for crate::validators::dependency::DependencyCycle {
        fn fmt_field(&self) -> String {
            self.0.join(" -> ")
        }
    }

    impl ViolationFieldFmt for Vec<(PathBuf, usize)> {
        fn fmt_field(&self) -> String {
            self.iter()
                .map(|(p, n)| format!("{}:{}", p.display(), n))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    impl ViolationFieldFmt for Vec<(PathBuf, usize, String)> {
        fn fmt_field(&self) -> String {
            self.iter()
                .map(|(p, n, s)| format!("{}:{}:{}", p.display(), n, s))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    impl ViolationFieldFmt for Vec<(PathBuf, String)> {
        fn fmt_field(&self) -> String {
            self.iter()
                .map(|(p, s)| format!("{}:{}", p.display(), s))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

/// Macro to implement the `Validator` trait for types that have a `validate_all()` method.
///
/// This eliminates the boilerplate `impl Validator` block that was repeated
/// in 12+ validator implementations. Each validator only needs to provide
/// a name and description; the `validate()` method delegates to `validate_all()`.
///
/// # Example
///
/// ```text
/// impl_validator!(PatternValidator, "patterns", "Validates code patterns (DI, async traits, error handling)");
/// ```
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
            ) -> anyhow::Result<Vec<Box<dyn $crate::traits::violation::Violation>>> {
                let violations = self.validate_all()?;
                Ok(violations
                    .into_iter()
                    .map(|v| Box::new(v) as Box<dyn $crate::traits::violation::Violation>)
                    .collect())
            }
        }
    };
}
