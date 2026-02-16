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
