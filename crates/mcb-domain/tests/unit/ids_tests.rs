//! Tests for strong-typed ID value objects

use mcb_domain::value_objects::ids::{
    ChunkId, CollectionId, ObservationId, OperationId, RepositoryId, SessionId,
};

#[test]
fn test_collection_id_creation() {
    let id = CollectionId::new("test-collection");
    assert_eq!(id.as_str(), "test-collection");
}

#[test]
fn test_collection_id_from_string() {
    let id = CollectionId::from("test".to_string());
    assert_eq!(id.as_str(), "test");
}

#[test]
fn test_collection_id_from_str() {
    let id = CollectionId::from("test");
    assert_eq!(id.as_str(), "test");
}

#[test]
fn test_collection_id_display() {
    let id = CollectionId::new("display-test");
    assert_eq!(id.to_string(), "display-test");
}

#[test]
fn test_chunk_id_creation() {
    let id = ChunkId::new("chunk-1");
    assert_eq!(id.as_str(), "chunk-1");
}

#[test]
fn test_repository_id_creation() {
    let id = RepositoryId::new("repo-1");
    assert_eq!(id.as_str(), "repo-1");
}

#[test]
fn test_session_id_creation() {
    let id = SessionId::new("session-1");
    assert_eq!(id.as_str(), "session-1");
}

#[test]
fn test_observation_id_creation() {
    let id = ObservationId::new("obs-1");
    assert_eq!(id.as_str(), "obs-1");
}

#[test]
fn test_operation_id_creation() {
    let id = OperationId::new("op-1");
    assert_eq!(id.as_str(), "op-1");
}

#[test]
fn test_id_equality() {
    let id1 = CollectionId::new("test");
    let id2 = CollectionId::new("test");
    assert_eq!(id1, id2);
}

#[test]
fn test_id_into_string() {
    let id = CollectionId::new("test");
    let s: String = id.into();
    assert_eq!(s, "test");
}

#[test]
fn test_id_as_ref() {
    let id = CollectionId::new("test");
    let s: &str = id.as_ref();
    assert_eq!(s, "test");
}
