//! Server macros organized by usage context.
//!
//! Sub-modules group macros by the server area they serve:
//! - [`args`]: Tool argument derive macros (used by `args/`)
//! - [`dispatch`]: Entity handler dispatch macros (used by `handlers/entities/`)
//! - [`handlers`]: Field extraction and validation macros (used by `handlers/`)

#[macro_use]
mod args;
#[macro_use]
mod dispatch;
#[macro_use]
mod handlers;
