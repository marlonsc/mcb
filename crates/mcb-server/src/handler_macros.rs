#[macro_export]
// TODO(NAME006): CA naming: Handler file outside handlers/ directory - Move to handlers/, admin/, or tools/
/// Dispatches `(action, resource)` pairs to handler expressions with an optional fallback.
macro_rules! entity_crud_dispatch {
    (
        action = $action:expr,
        resource = $resource:expr,
        fallback = |$unsupported_action:ident, $unsupported_resource:ident| $fallback:expr,
        { $($arms:tt)* }
    ) => {
        match ($action, $resource) {
            $($arms)*
            ($unsupported_action, $unsupported_resource) => $fallback,
        }
    };
    (
        action = $action:expr,
        resource = $resource:expr,
        { $($arms:tt)* }
    ) => {
        match ($action, $resource) {
            $($arms)*
            (action, resource) => Err(rmcp::model::ErrorData::invalid_params(
                format!("Unsupported action {:?} for resource {:?}", action, resource),
                None,
            )),
        }
    };
}
