use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsBranch {
    id: String,
    name: String,
    head_commit: String,
    is_default: bool,
    upstream: Option<String>,
}

impl VcsBranch {
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn head_commit(&self) -> &str {
        &self.head_commit
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn upstream(&self) -> Option<&str> {
        self.upstream.as_deref()
    }
}
