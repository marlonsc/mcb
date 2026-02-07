use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactType {
    Module,
    Struct,
    Function,
    Import,
    Trait,
    Impl,
    Macro,
    Constant,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub name: String,
    pub fact_type: FactType,
    pub location: Location,
    pub attributes: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
    // For graph building: unique identifier
    pub id: String,
    // Parent ID (e.g., module for a struct)
    pub parent_id: Option<String>,
}

impl Fact {
    pub fn new(
        name: String,
        fact_type: FactType,
        location: Location,
        parent_id: Option<String>,
    ) -> Self {
        let id = if let Some(parent) = &parent_id {
            format!("{}::{}", parent, name)
        } else {
            name.clone()
        };

        Self {
            name,
            fact_type,
            location,
            attributes: HashMap::new(),
            metadata: HashMap::new(),
            id,
            parent_id,
        }
    }

    pub fn add_attribute(&mut self, key: &str, value: String) {
        self.attributes.insert(key.to_string(), value);
    }
}
