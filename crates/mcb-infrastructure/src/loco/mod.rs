//! LocoBridge — Composition root for Loco framework integration.
//!
//! Extracts framework resources (DB, cache, config) from LocoAppContext
//! and feeds them into MCB's DI system via DomainServicesFactory.

pub mod bridge;
pub use bridge::LocoBridge;
