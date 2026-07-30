//! Server macros organized by usage context.
//!
//! Sub-modules group macros by the server area they serve:
//! - [`accessors`]: Arc accessor generation (used by `mcp_server`)
//! - [`args`]: Tool argument derive macros (used by `args/`)
//! - [`dispatch`]: Entity handler dispatch macros (used by `handlers/entities/`)
//! - [`handlers`]: Field extraction and validation macros (used by `handlers/`)
//! - [`registry`]: Tool registration macros (used by `tools/registry`)

#[macro_use]
mod accessors;
#[macro_use]
mod args;
#[macro_use]
mod dispatch;
#[macro_use]
mod handlers;
#[macro_use]
mod registry;
