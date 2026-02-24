//! Event bus adapters owned by the infrastructure layer.

/// In-process event bus implementation used by Loco wiring.
pub mod loco;

pub use loco::LocoEventBusProvider;
