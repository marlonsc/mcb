//! SeaORM-based database layer: deterministic migrations and entity definitions.
#[allow(missing_docs)]
pub mod entities;
pub mod migration;

/// Domain ↔ SeaORM entity conversion layer (From impls).
pub mod conversions;
