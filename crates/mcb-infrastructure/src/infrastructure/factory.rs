/// Create the default in-process event bus for standalone contexts.
///
/// Uses `TokioEventBusProvider` — the same implementation resolved by the DI
/// bootstrap. Exported so that `mcb-server` can build a minimal `AdminState`
/// without a full `AppContext` while avoiding Null Object patterns.
#[must_use]
pub fn default_event_bus() -> std::sync::Arc<dyn mcb_domain::ports::infrastructure::EventBusProvider>
{
    std::sync::Arc::new(mcb_providers::events::TokioEventBusProvider::new())
}
