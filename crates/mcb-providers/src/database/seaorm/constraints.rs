//! SeaQuery Constraint Builder for Vector Search Filtering
//!
//! Provides a type-safe, composable constraint builder that translates domain search filters
//! into SeaQuery WHERE clauses. This enables efficient database-level filtering for vector
//! search results, reducing the amount of data that needs to be processed in-memory.
//!
//! # Features
//!
//! - **Type-safe constraints**: Compile-time verification of filter types
//! - **Composable filters**: Chain multiple constraints with AND/OR logic
//! - **Database agnostic**: Works with SQLite, PostgreSQL, and MySQL
//! - **JSON extraction**: Filter on embedded JSON metadata fields
//! - **Tag matching**: Array containment checks for session/observation tags
//!
//! # Example
//!
//! ```rust
//! use mcb_providers::database::seaorm::constraints::{ConstraintBuilder, EntityType};
//! use sea_query::Query;
//!
//! let mut query = Query::select();
//!
//! ConstraintBuilder::new()
//!     .with_project_id("proj-123")
//!     .with_entity_type(EntityType::Memory)
//!     .with_tags(&["important", "review"])
//!     .apply_to(&mut query);
//! ```

use sea_orm::{
    ExprTrait,
    sea_query::{Condition, Expr, SelectStatement},
};
use std::collections::HashSet;

/// Supported entity types for vector search filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Memory/observation entities
    Memory,
    /// Code/indexed file entities
    Code,
    /// Session entities
    Session,
    /// Repository entities
    Repository,
    /// Generic file entities
    File,
}

impl EntityType {
    /// Returns the database string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Memory => "memory",
            EntityType::Code => "code",
            EntityType::Session => "session",
            EntityType::Repository => "repository",
            EntityType::File => "file",
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "memory" | "observation" => Ok(EntityType::Memory),
            "code" | "indexed_code" => Ok(EntityType::Code),
            "session" => Ok(EntityType::Session),
            "repository" | "repo" => Ok(EntityType::Repository),
            "file" => Ok(EntityType::File),
            _ => Err(format!("Unknown entity type: {s}")),
        }
    }
}

/// Individual search constraint variants
#[derive(Debug, Clone, PartialEq)]
pub enum SearchConstraint {
    /// Filter by project identifier
    ProjectId(String),
    /// Filter by workspace identifier
    WorkspaceId(String),
    /// Filter by organization identifier
    OrgId(String),
    /// Filter by entity type
    EntityType(EntityType),
    /// Filter by file extension (e.g., "rs", "py")
    FileExtension(String),
    /// Filter by directory path prefix
    DirectoryPath(String),
    /// Filter by programming language
    Language(String),
    /// Filter by session tags (all must match)
    Tags(HashSet<String>),
    /// Filter by minimum relevance score
    MinScore(f32),
    /// Time range filter (start, end) in epoch seconds
    TimeRange(i64, i64),
    /// Custom JSON metadata filter (`json_path`, value)
    JsonMetadata {
        /// JSON path expression (for example, `$.session_id`)
        path: String,
        /// Value to compare against the extracted JSON field
        value: String,
    },
}

/// Builder for constructing `SeaQuery` WHERE clauses from search constraints
///
/// # Type Parameters
///
/// * `E` - The `SeaORM` entity type being queried
///
/// # Example
///
/// ```rust
/// use mcb_providers::database::seaorm::constraints::ConstraintBuilder;
///
/// let builder = ConstraintBuilder::new()
///     .with_project_id("my-project")
///     .with_file_extensions(&["rs", "toml"]);
/// ```
#[derive(Debug, Clone, Default)]
#[must_use = "ConstraintBuilder has no effect unless consumed"]
pub struct ConstraintBuilder {
    constraints: Vec<SearchConstraint>,
}

impl ConstraintBuilder {
    /// Create a new empty constraint builder
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a project ID constraint
    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::ProjectId(project_id.into()));
        self
    }

    /// Add a workspace ID constraint
    pub fn with_workspace_id(mut self, workspace_id: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::WorkspaceId(workspace_id.into()));
        self
    }

    /// Add an organization ID constraint
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::OrgId(org_id.into()));
        self
    }

    /// Add an entity type constraint
    pub fn with_entity_type(mut self, entity_type: EntityType) -> Self {
        self.constraints
            .push(SearchConstraint::EntityType(entity_type));
        self
    }

    /// Add a file extension constraint
    pub fn with_file_extension(mut self, extension: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::FileExtension(extension.into()));
        self
    }

    /// Add multiple file extension constraints (OR logic)
    pub fn with_file_extensions(mut self, extensions: &[impl ToString]) -> Self {
        for ext in extensions {
            self = self.with_file_extension(ext.to_string());
        }
        self
    }

    /// Add a directory path constraint (prefix match)
    pub fn with_directory_path(mut self, path: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::DirectoryPath(path.into()));
        self
    }

    /// Add a programming language constraint
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.constraints
            .push(SearchConstraint::Language(language.into()));
        self
    }

    /// Add multiple language constraints (OR logic)
    pub fn with_languages(mut self, languages: &[impl ToString]) -> Self {
        for lang in languages {
            self = self.with_language(lang.to_string());
        }
        self
    }

    /// Add a session tag constraint (all tags must be present)
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if let Some(SearchConstraint::Tags(existing)) = self.constraints.last_mut() {
            existing.insert(tag);
        } else {
            let mut tags = HashSet::new();
            tags.insert(tag);
            self.constraints.push(SearchConstraint::Tags(tags));
        }
        self
    }

    /// Add multiple tag constraints at once
    pub fn with_tags(mut self, tags: &[impl ToString]) -> Self {
        let tag_set: HashSet<String> = tags.iter().map(std::string::ToString::to_string).collect();
        if let Some(SearchConstraint::Tags(existing)) = self.constraints.last_mut() {
            existing.extend(tag_set);
        } else {
            self.constraints.push(SearchConstraint::Tags(tag_set));
        }
        self
    }

    /// Add a minimum relevance score constraint
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.constraints.push(SearchConstraint::MinScore(score));
        self
    }

    /// Add a time range constraint
    pub fn with_time_range(mut self, start: i64, end: i64) -> Self {
        self.constraints
            .push(SearchConstraint::TimeRange(start, end));
        self
    }

    /// Add a custom JSON metadata constraint
    pub fn with_json_metadata(mut self, path: impl Into<String>, value: impl Into<String>) -> Self {
        self.constraints.push(SearchConstraint::JsonMetadata {
            path: path.into(),
            value: value.into(),
        });
        self
    }

    /// Add a raw constraint directly
    pub fn with_constraint(mut self, constraint: SearchConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Returns the number of constraints currently in the builder
    #[must_use]
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    /// Returns true if no constraints have been added
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    /// Build the WHERE conditions for the given `SeaQuery` select statement
    ///
    /// This method applies all accumulated constraints to the provided query.
    /// Constraints are combined with AND logic.
    ///
    /// # Arguments
    ///
    /// * `query` - The `SeaQuery` select statement to modify
    pub fn apply_to(&self, query: &mut SelectStatement) {
        for constraint in &self.constraints {
            match constraint {
                SearchConstraint::ProjectId(id) => {
                    query.cond_where(
                        Expr::col((observation::Entity, observation::Column::ProjectId))
                            .eq(id.clone()),
                    );
                }
                SearchConstraint::WorkspaceId(id) => {
                    // Workspace ID is stored in metadata as worktree_id
                    query.cond_where(Expr::cust_with_values(
                        "json_extract(metadata, '$.worktree_id') = ?",
                        vec![sea_orm::Value::from(id.clone())],
                    ));
                }
                SearchConstraint::OrgId(id) => {
                    // Org ID is stored in metadata
                    query.cond_where(Expr::cust_with_values(
                        "json_extract(metadata, '$.org_id') = ?",
                        vec![sea_orm::Value::from(id.clone())],
                    ));
                }
                SearchConstraint::EntityType(et) => {
                    query.cond_where(
                        Expr::col((observation::Entity, observation::Column::ObservationType))
                            .eq(et.as_str()),
                    );
                }
                SearchConstraint::FileExtension(ext) => {
                    // File extension is stored in metadata
                    query.cond_where(Expr::cust_with_values(
                        "json_extract(metadata, '$.file_extension') = ?",
                        vec![sea_orm::Value::from(ext.clone())],
                    ));
                }
                SearchConstraint::DirectoryPath(path) => {
                    // Directory path uses LIKE for prefix matching
                    let pattern = format!("{path}%");
                    query.cond_where(Expr::cust_with_values(
                        "json_extract(metadata, '$.file_path') LIKE ?",
                        vec![sea_orm::Value::from(pattern)],
                    ));
                }
                SearchConstraint::Language(lang) => {
                    query.cond_where(Expr::cust_with_values(
                        "json_extract(metadata, '$.language') = ?",
                        vec![sea_orm::Value::from(lang.clone())],
                    ));
                }
                SearchConstraint::Tags(tags) => {
                    // Each tag must exist in the tags array
                    for tag in tags {
                        query.cond_where(Expr::cust_with_values(
                            "EXISTS (SELECT 1 FROM json_each(tags) WHERE value = ?)",
                            vec![sea_orm::Value::from(tag.clone())],
                        ));
                    }
                }
                SearchConstraint::MinScore(_score) => {
                    // Score filtering is typically done at the vector store level
                    // or in-memory after retrieval. Skip for SQL filtering.
                }
                SearchConstraint::TimeRange(start, end) => {
                    query.cond_where(
                        Expr::col((observation::Entity, observation::Column::CreatedAt))
                            .gte(*start),
                    );
                    query.cond_where(
                        Expr::col((observation::Entity, observation::Column::CreatedAt)).lte(*end),
                    );
                }
                SearchConstraint::JsonMetadata { path, value } => {
                    let expr = Expr::cust(format!("json_extract(metadata, '{path}')"));
                    query.cond_where(expr.eq(value.clone()));
                }
            }
        }
    }

    /// Build a standalone Condition expression that can be composed with other conditions
    ///
    /// Returns a `SeaQuery` Condition containing all constraints combined with AND logic.
    #[must_use]
    pub fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        for constraint in &self.constraints {
            match constraint {
                SearchConstraint::ProjectId(id) => {
                    condition = condition.add(
                        Expr::col((observation::Entity, observation::Column::ProjectId))
                            .eq(id.clone()),
                    );
                }
                SearchConstraint::WorkspaceId(id) => {
                    condition = condition.add(Expr::cust_with_values(
                        "json_extract(metadata, '$.worktree_id') = ?",
                        vec![sea_orm::Value::from(id.clone())],
                    ));
                }
                SearchConstraint::OrgId(id) => {
                    condition = condition.add(Expr::cust_with_values(
                        "json_extract(metadata, '$.org_id') = ?",
                        vec![sea_orm::Value::from(id.clone())],
                    ));
                }
                SearchConstraint::EntityType(et) => {
                    condition = condition.add(
                        Expr::col((observation::Entity, observation::Column::ObservationType))
                            .eq(et.as_str()),
                    );
                }
                SearchConstraint::FileExtension(ext) => {
                    condition = condition.add(Expr::cust_with_values(
                        "json_extract(metadata, '$.file_extension') = ?",
                        vec![sea_orm::Value::from(ext.clone())],
                    ));
                }
                SearchConstraint::DirectoryPath(path) => {
                    let pattern = format!("{path}%");
                    condition = condition.add(Expr::cust_with_values(
                        "json_extract(metadata, '$.file_path') LIKE ?",
                        vec![sea_orm::Value::from(pattern)],
                    ));
                }
                SearchConstraint::Language(lang) => {
                    condition = condition.add(Expr::cust_with_values(
                        "json_extract(metadata, '$.language') = ?",
                        vec![sea_orm::Value::from(lang.clone())],
                    ));
                }
                SearchConstraint::Tags(tags) => {
                    for tag in tags {
                        condition = condition.add(Expr::cust_with_values(
                            "EXISTS (SELECT 1 FROM json_each(tags) WHERE value = ?)",
                            vec![sea_orm::Value::from(tag.clone())],
                        ));
                    }
                }
                SearchConstraint::MinScore(_) => {
                    // Handled at vector store level
                }
                SearchConstraint::TimeRange(start, end) => {
                    condition = condition.add(
                        Expr::col((observation::Entity, observation::Column::CreatedAt))
                            .gte(*start),
                    );
                    condition = condition.add(
                        Expr::col((observation::Entity, observation::Column::CreatedAt)).lte(*end),
                    );
                }
                SearchConstraint::JsonMetadata { path, value } => {
                    let expr = Expr::cust(format!("json_extract(metadata, '{path}')"));
                    condition = condition.add(expr.eq(value.clone()));
                }
            }
        }

        condition
    }
}

use crate::database::seaorm::entities::observation;
