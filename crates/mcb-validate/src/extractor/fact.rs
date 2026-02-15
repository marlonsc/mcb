//! Code fact data types.
//!
//! Defines the [`Fact`], [`FactType`], and [`Location`] types used to represent
//! extracted information from source code ASTs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Categorisation of a code fact (module, struct, function, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactType {
    /// A Rust module declaration.
    Module,
    /// A struct definition.
    Struct,
    /// A function definition.
    Function,
    /// A `use` import declaration.
    Import,
    /// A trait definition.
    Trait,
    /// An `impl` block.
    Impl,
    /// A macro definition.
    Macro,
    /// A constant or static item.
    Constant,
    /// An unrecognised AST node kind.
    Unknown,
}

/// Source-code location for a fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Path to the file containing this fact.
    pub file_path: PathBuf,
    /// 1-based start line number.
    pub start_line: usize,
    /// 1-based end line number.
    pub end_line: usize,
    /// 1-based start column.
    pub start_column: usize,
    /// 1-based end column.
    pub end_column: usize,
}

/// A single extracted code fact (module, struct, function, import, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Human-readable name of the fact (e.g. struct name, import path).
    pub name: String,
    /// The category of this fact.
    pub fact_type: FactType,
    /// Where in the source file this fact was found.
    pub location: Location,
    /// Key-value attributes attached during extraction.
    pub attributes: HashMap<String, String>,
    /// Arbitrary JSON metadata for downstream consumers.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Unique identifier used for graph construction.
    pub id: String,
    /// Optional parent identifier (e.g. a module containing a struct).
    pub parent_id: Option<String>,
}

impl Fact {
    /// Create a new `Fact` with an auto-generated `id`.
    ///
    /// The `id` is derived from the optional `parent_id` and `name`:
    /// `"{parent_id}::{name}"` when a parent is present, otherwise just `name`.
    #[must_use]
    pub fn new(
        name: String,
        fact_type: FactType,
        location: Location,
        parent_id: Option<String>,
    ) -> Self {
        let id = if let Some(parent) = &parent_id {
            format!("{parent}::{name}")
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

    /// Attach a key-value attribute to this fact.
    pub fn add_attribute(&mut self, key: &str, value: String) {
        self.attributes.insert(key.to_owned(), value);
    }
}
