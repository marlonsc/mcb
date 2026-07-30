//! Declarative macros for the domain layer.
//!
//! Groups reusable `macro_rules!` definitions for entity boilerplate, logging,
//! port trait scaffolding, registry generation, service helpers, and test
//! fixtures. Each submodule is `#[macro_use]` so its macros are available
//! crate-wide.

#[macro_use]
mod entities;
#[macro_use]
mod logging;
#[macro_use]
mod ports;
#[macro_use]
mod registry;
#[macro_use]
mod services;
#[macro_use]
mod testing;
