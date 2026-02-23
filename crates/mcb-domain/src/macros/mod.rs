//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Domain macros organized by usage context.
//!
//! Sub-modules group macros by the domain area they serve:
//! - [`entities`]: Entity and value-object type macros
//! - [`ports`]: Port trait definition macros
//! - [`schema`]: DDL schema builder macros
//! - [`registry`]: Provider registry infrastructure macros

#[macro_use]
mod entities;
#[macro_use]
mod logging;
#[macro_use]
mod ports;
#[macro_use]
mod schema;
#[macro_use]
mod registry;
