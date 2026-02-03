//! Database access for infrastructure.
//!
//! Providers supply connection pools to repositories; repositories do not
//! open SQLite connections directly.

pub mod memory_provider;

pub use memory_provider::MemoryDatabaseProvider;
