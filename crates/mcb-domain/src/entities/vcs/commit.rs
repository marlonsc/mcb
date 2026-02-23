//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
//! VCS Commit Entity
//!
//! This module defines the `VcsCommit` entity, representing a single commit
//! within a Version Control System. It captures essential metadata such as
//! hash, message, author information, and parentage.

use serde::{Deserialize, Serialize};

/// `VcsCommit` entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsCommit {
    id: String,
    hash: String,
    message: String,
    author: String,
    author_email: String,
    timestamp: i64,
    parent_hashes: Vec<String>,
}

/// `VcsCommitInput` struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsCommitInput {
    /// Unique identifier
    pub id: String,
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Author name
    pub author: String,
    /// Author email
    pub author_email: String,
    /// Unix timestamp
    pub timestamp: i64,
    /// Parent commit hashes
    pub parent_hashes: Vec<String>,
}

impl VcsCommit {
    /// Creates a new `VcsCommit`.
    #[must_use]
    pub fn new(input: VcsCommitInput) -> Self {
        Self {
            id: input.id,
            hash: input.hash,
            message: input.message,
            author: input.author,
            author_email: input.author_email,
            timestamp: input.timestamp,
            parent_hashes: input.parent_hashes,
        }
    }

    /// Performs the id operation.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Performs the hash operation.
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Performs the message operation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Performs the author operation.
    #[must_use]
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Performs the author email operation.
    #[must_use]
    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    /// Performs the timestamp operation.
    #[must_use]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Performs the parent hashes operation.
    #[must_use]
    pub fn parent_hashes(&self) -> &[String] {
        &self.parent_hashes
    }
}
