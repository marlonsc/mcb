use serde::{Deserialize, Serialize};

/// VcsBranch entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsBranch {
    id: String,
    name: String,
    head_commit: String,
    is_default: bool,
    upstream: Option<String>,
}

impl VcsBranch {
    /// Creates a new instance.
    pub fn new(
        id: String,
        name: String,
        head_commit: String,
        is_default: bool,
        upstream: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            head_commit,
            is_default,
            upstream,
        }
    }

    /// Performs the id operation.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Performs the name operation.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Performs the head commit operation.
    pub fn head_commit(&self) -> &str {
        &self.head_commit
    }

    /// Performs the is default operation.
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Performs the upstream operation.
    pub fn upstream(&self) -> Option<&str> {
        self.upstream.as_deref()
    }
}
