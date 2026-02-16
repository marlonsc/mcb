//! Repository Interfaces (re-exports from [`crate::ports::repositories`]).
//!
//! This module provides backward-compatible access to repository port traits.
//! The canonical definitions live in [`crate::ports::repositories`].

pub use crate::ports::repositories::chunk;
pub use crate::ports::repositories::search;

pub use crate::ports::repositories::{ChunkRepository, RepositoryStats};
pub use crate::ports::repositories::{SearchRepository, SearchStats};
