use serde::{Deserialize, Serialize};

/// VcsCommit entity.
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
    /// Creates a new instance.
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
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Performs the hash operation.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Performs the message operation.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Performs the author operation.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Performs the author email operation.
    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    /// Performs the timestamp operation.
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Performs the parent hashes operation.
    pub fn parent_hashes(&self) -> &[String] {
        &self.parent_hashes
    }
}
