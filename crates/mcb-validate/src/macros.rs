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
