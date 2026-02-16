//! Violation enum definition macro with automatic trait implementations.
//!
//! Used by 37+ validator files to define strongly-typed violation enums
//! with auto-generated `Display`, `Violation`, file/line extraction, and suggestion support.

/// Defines violation enums and auto-generates `Display` and `Violation` implementations.
/// Macro to define violation enums with automatic trait implementations.
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
        define_violations! {
            @emit_variants
            no_display,
            dynamic_severity,
            $category,
            $vis enum $name {
                $(
                    $(#[doc = $doc])*
                    #[violation(id = $id, severity = $severity $(, message = $msg)? $(, suggestion = $suggestion)?)]
                    $variant { $( $field : $field_ty ),* }
                ),*
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
        define_violations! {
            @emit_variants
            with_display,
            dynamic_severity,
            $category,
            $vis enum $name {
                $(
                    $(#[doc = $doc])*
                    #[violation(id = $id, severity = $severity $(, message = $msg)? $(, suggestion = $suggestion)?)]
                    $variant { $( $field : $field_ty ),* }
                ),*
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
        define_violations! {
            @emit_variants
            no_display,
            static_severity,
            $category,
            $vis enum $name {
                $(
                    $(#[doc = $doc])*
                    #[violation(id = $id, severity = $severity $(, message = $msg)? $(, suggestion = $suggestion)?)]
                    $variant { $( $field : $field_ty ),* }
                ),*
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
        define_violations! {
            @emit_variants
            with_display,
            static_severity,
            $category,
            $vis enum $name {
                $(
                    $(#[doc = $doc])*
                    #[violation(id = $id, severity = $severity $(, message = $msg)? $(, suggestion = $suggestion)?)]
                    $variant { $( $field : $field_ty ),* }
                ),*
            }
        }
    };

    (@emit_variants
        $display:tt,
        $severity_mode:ident,
        $category:expr,
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                #[violation(id = $id:literal, severity = $severity:ident $(, message = $msg:literal)? $(, suggestion = $suggestion:literal)?)]
                $variant:ident { $( $field:ident : $field_ty:ty ),* }
            ),* $(,)?
        }
    ) => {
        define_violations! {
            @enum_def
            $vis enum $name {
                $(
                    $(#[doc = $doc])*
                    $variant { $( $field : $field_ty ),* }
                ),*
            }
        }

        define_violations! {
            @maybe_display
            $display,
            $name {
                $(
                    $variant { $( $field : $field_ty ),* }
                    $(=> $msg)?
                ),*
            }
        }

        define_violations! {
            @impl_violation
            $severity_mode,
            $category,
            $name {
                $(
                    #[violation(id = $id, severity = $severity $(, suggestion = $suggestion)?)]
                    $variant { $( $field : $field_ty ),* }
                ),*
            }
        }
    };

    (@maybe_display
        no_display,
        $name:ident {
            $( $variant:ident { $( $field:ident : $field_ty:ty ),* } $(=> $msg:literal)? ),* $(,)?
        }
    ) => {};

    (@maybe_display
        with_display,
        $name:ident {
            $( $variant:ident { $( $field:ident : $field_ty:ty ),* } $(=> $msg:literal)? ),* $(,)?
        }
    ) => {
        define_violations! {
            @impl_display
            $name {
                $(
                    $variant { $( $field : $field_ty ),* }
                    $(=> $msg)?
                ),*
            }
        }
    };

    (@enum_def
        $vis:vis enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                $variant:ident { $( $field:ident : $field_ty:ty ),* }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize)]
        #[doc = concat!("Violation types emitted by the [`", stringify!($name), "`] validator.")]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $variant {
                    $(
                        #[doc = concat!(" `", stringify!($field), "` value.")]
                        $field: $field_ty
                    ),*
                }
            ),*
        }
    };

    (@impl_display
        $name:ident {
            $(
                $variant:ident { $( $field:ident : $field_ty:ty ),* }
                $(=> $msg:literal)?
            ),* $(,)?
        }
    ) => {
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

    (@impl_violation
        dynamic_severity,
        $category:expr,
        $name:ident {
            $(
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident { $( $field:ident : $field_ty:ty ),* }
            ),* $(,)?
        }
    ) => {
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

            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_file_arm self, $variant, $( $field : $field_ty ),*) ),*
                }
            }

            fn line(&self) -> Option<usize> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_line_arm self, $variant, $( $field : $field_ty ),*) ),*
                }
            }

            fn suggestion(&self) -> Option<String> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_suggestion_arm self, $variant, $($suggestion,)? $( $field : $field_ty ),*) ),*
                }
            }
        }
    };

    (@impl_violation
        static_severity,
        $category:expr,
        $name:ident {
            $(
                #[violation(
                    id = $id:literal,
                    severity = $severity:ident
                    $(, suggestion = $suggestion:literal)?
                )]
                $variant:ident { $( $field:ident : $field_ty:ty ),* }
            ),* $(,)?
        }
    ) => {
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

            fn file(&self) -> Option<&std::path::PathBuf> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_file_arm self, $variant, $( $field : $field_ty ),*) ),*
                }
            }

            fn line(&self) -> Option<usize> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_line_arm self, $variant, $( $field : $field_ty ),*) ),*
                }
            }

            fn suggestion(&self) -> Option<String> {
                match self {
                    $( Self::$variant { .. } => define_violations!(@select_suggestion_arm self, $variant, $($suggestion,)? $( $field : $field_ty ),*) ),*
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

    (@select_file_arm $self:expr, $variant:ident, file : $file_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { file, .. } = $self {
            Some(file)
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, path : $path_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { path, .. } = $self {
            Some(path)
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, location : $location_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { location, .. } = $self {
            Some(location)
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, source_file : $source_file_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { source_file, .. } = $self {
            Some(source_file)
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, referencing_file : $referencing_file_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { referencing_file, .. } = $self {
            Some(referencing_file)
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, locations : $loc_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { locations, .. } = $self {
            locations
                .first()
                .map(|loc| $crate::violation_macro::ExtractFilePath::extract_file_path(loc))
        } else {
            None
        }
    };
    (@select_file_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty, $( $rest:ident : $rest_ty:ty ),+ ) => {
        define_violations!(@select_file_arm $self, $variant, $( $rest : $rest_ty ),+)
    };
    (@select_file_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty) => {
        None
    };
    (@select_file_arm $self:expr, $variant:ident,) => {
        None
    };

    (@select_line_arm $self:expr, $variant:ident, line : $line_ty:ty $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { line, .. } = $self {
            Some(*line)
        } else {
            None
        }
    };
    (@select_line_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty, $( $rest:ident : $rest_ty:ty ),+ ) => {
        define_violations!(@select_line_arm $self, $variant, $( $rest : $rest_ty ),+)
    };
    (@select_line_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty) => {
        None
    };
    (@select_line_arm $self:expr, $variant:ident,) => {
        None
    };

    (@select_suggestion_arm $self:expr, $variant:ident, $suggestion:literal, $( $field:ident : $field_ty:ty ),*) => {
        if let Self::$variant { $( $field ),* } = $self {
            Some(define_violations!(@render_template $suggestion, $( $field : $field_ty ),*))
        } else {
            None
        }
    };
    (@select_suggestion_arm $self:expr, $variant:ident, suggestion : String $(, $rest:ident : $rest_ty:ty )* ) => {
        if let Self::$variant { suggestion, .. } = $self {
            Some(suggestion.clone())
        } else {
            None
        }
    };
    (@select_suggestion_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty, $( $rest:ident : $rest_ty:ty ),+ ) => {
        define_violations!(@select_suggestion_arm $self, $variant, $( $rest : $rest_ty ),+)
    };
    (@select_suggestion_arm $self:expr, $variant:ident, $skip:ident : $skip_ty:ty) => {
        None
    };
    (@select_suggestion_arm $self:expr, $variant:ident,) => {
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
