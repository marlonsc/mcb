use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default)]
pub struct CommitBuilder {
    id: String,
    hash: String,
    message: String,
    author: String,
    author_email: String,
    timestamp: i64,
    parent_hashes: Vec<String>,
}

impl CommitBuilder {
    pub fn new(id: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            hash: hash.into(),
            ..Default::default()
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn author(mut self, author: impl Into<String>, email: impl Into<String>) -> Self {
        self.author = author.into();
        self.author_email = email.into();
        self
    }

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn parents(mut self, parents: Vec<String>) -> Self {
        self.parent_hashes = parents;
        self
    }

    pub fn build(self) -> VcsCommit {
        VcsCommit {
            id: self.id,
            hash: self.hash,
            message: self.message,
            author: self.author,
            author_email: self.author_email,
            timestamp: self.timestamp,
            parent_hashes: self.parent_hashes,
        }
    }
}

impl VcsCommit {
    pub fn builder(id: impl Into<String>, hash: impl Into<String>) -> CommitBuilder {
        CommitBuilder::new(id, hash)
    }

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
