//! `LocoBridge` — Composition root for Loco framework integration.
//!
//! Extracts framework resources (DB, cache, config) from `LocoAppContext`
//! and feeds them into MCB's DI system via `DomainServicesFactory`.

/// Bridge between Loco framework `AppContext` and MCB's DI system.
pub mod bridge;
pub use bridge::LocoBridge;
