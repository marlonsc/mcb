//!
//! GraphQL schema limits used by Seaography.

/// Maximum depth of the GraphQL query tree.
pub const SCHEMA_DEPTH: i32 = 15;

/// Maximum complexity score for GraphQL queries.
pub const SCHEMA_COMPLEXITY: i32 = 250;
