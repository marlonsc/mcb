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

impl VcsCommit {
    /// Creates a new `VcsCommit` instance.
    ///
    /// # Parameters
    /// TODO(qlty): Function with many parameters (count = 7).
    #[must_use]
    pub fn new(
        id: String,
        hash: String,
        message: String,
        author: String,
        author_email: String,
        timestamp: i64,
        parent_hashes: Vec<String>,
    ) -> Self {
        Self {
            id,
            hash,
            message,
            author,
            author_email,
            timestamp,
            parent_hashes,
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
