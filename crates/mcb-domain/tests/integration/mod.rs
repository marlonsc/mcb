//! Integration test suite for mcb-domain
//!
//! Run with: `cargo test -p mcb-domain --test integration`

#[path = "entity_value_object_integration.rs"]
mod entity_value_object_integration;
mod file_tree_node_integration;
#[path = "semantic_search_workflow.rs"]
mod semantic_search_workflow;
