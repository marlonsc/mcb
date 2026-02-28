//! Tests for constraint builder.

use mcb_providers::database::seaorm::constraints::{ConstraintBuilder, EntityType};
use std::str::FromStr;

#[test]
fn test_entity_type_from_str() {
    assert_eq!(EntityType::from_str("memory").unwrap(), EntityType::Memory);
    assert_eq!(EntityType::from_str("CODE").unwrap(), EntityType::Code);
    assert!(EntityType::from_str("unknown").is_err());
}

#[test]
fn test_constraint_builder_chaining() {
    let builder = ConstraintBuilder::new()
        .with_project_id("proj-123")
        .with_entity_type(EntityType::Memory)
        .with_file_extension("rs")
        .with_tag("important");

    assert_eq!(builder.len(), 4);
}

#[test]
fn test_constraint_builder_tags() {
    let builder = ConstraintBuilder::new().with_tags(&["tag1", "tag2", "tag3"]);

    assert_eq!(builder.len(), 1);
    // We can't directly inspect the constraints field as it's private,
    // but we can verify the builder was created and has the expected length
}

#[test]
fn test_empty_builder() {
    let builder = ConstraintBuilder::new();
    assert!(builder.is_empty());
    assert_eq!(builder.len(), 0);
}

#[test]
fn test_build_condition_returns_all() {
    let builder = ConstraintBuilder::new()
        .with_project_id("proj-123")
        .with_min_score(0.8);

    let _condition = builder.build_condition();
    // Condition should have 1 entry (MinScore is skipped for SQL)
    // We can't easily inspect the condition contents, but we can verify it builds
    assert!(!builder.is_empty());
}
