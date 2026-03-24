//!
//! **Documentation**: [docs/modules/providers.md](../../../docs/modules/providers.md)
//!
//! Centralized macros for provider implementations.
//!
//! Submodules group macros by provider category; exported macros are available
//! at crate root via `#[macro_export]` or in scope via `#[macro_use]`.
//!
//! - [`embedding`]: HTTP embedding struct/trait/linkme registration
//! - [`language`]: Language processor delegation and generation
//! - [`vector_store`]: Vector store actor message and linkme registration

#[macro_use]
pub mod embedding;
#[macro_use]
pub mod language;
#[macro_use]
pub mod vector_store;
#[macro_use]
pub mod conversion;
