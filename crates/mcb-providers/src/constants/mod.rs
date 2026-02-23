//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Provider Constants
//!
//! Constants specific to provider implementations. These are separated from
//! domain constants (which live in mcb-domain) and infrastructure constants.

/// Cache constants (`Redis`, etc.)
pub mod cache;
/// Database constants (`SQL`, etc.)
pub mod database;
/// Embedding constants (`OpenAI`, `VoyageAI`, etc.)
pub mod embedding;
/// Event bus constants (`NATS`, `Tokio`)
pub mod events;
/// HTTP client constants
pub mod http;
/// Language processing constants
pub mod language;
/// Retry configuration constants
pub mod retry;
/// Vector store constants (`Milvus`, `Qdrant`, etc.)
pub mod vector_store;

pub use self::cache::*;
pub use self::database::*;
pub use self::embedding::*;
pub use self::events::*;
pub use self::http::*;
pub use self::language::*;
pub use self::retry::*;
pub use self::vector_store::*;
