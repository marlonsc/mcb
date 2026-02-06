//! Infrastructure layer constants. Domain-specific constants are in `mcb_domain::constants`.

pub mod ast;
pub mod auth;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod db;
pub mod embedding;
pub mod error_msgs;
pub mod events;
pub mod fs;
pub mod health;
pub mod http;
pub mod lang;
pub mod limits;
pub mod metadata;
pub mod metrics;
pub mod network;
pub mod ops;
pub mod process;
pub mod resilience;
pub mod search;
pub mod sync;

// Re-export common constants for backward compatibility
pub use auth::*;
pub use cache::*;
pub use config::*;
pub use db::*;
pub use events::*;
pub use fs::*;
pub use http::*;
pub use lang::*;
pub use metrics::*;
pub use network::*;

// Re-export domain constants for convenience
pub use mcb_domain::constants::*;
