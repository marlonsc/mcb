use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsCommit {
    pub id: String,
    hash: String,
    pub message: String,
    author: String,
    pub author_email: String,
    pub timestamp: i64,
    parent_hashes: Vec<String>,
}

impl VcsCommit {
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn parent_hashes(&self) -> &[String] {
        &self.parent_hashes
    }
}
