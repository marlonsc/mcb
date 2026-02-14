//! Tests for strong-typed ID value objects

use mcb_domain::value_objects::ids::{
    ChunkId, CollectionId, ObservationId, OperationId, RepositoryId, SessionId,
};
use rstest::rstest;

#[rstest]
#[case(CollectionId::new("test-collection").as_str().to_string(), "test-collection")]
#[case(CollectionId::from("test".to_string()).as_str().to_string(), "test")]
#[case(CollectionId::from("test").as_str().to_string(), "test")]
#[case(CollectionId::new("display-test").to_string(), "display-test")]
#[case(ChunkId::new("chunk-1").as_str().to_string(), "chunk-1")]
#[case(RepositoryId::new("repo-1").as_str().to_string(), "repo-1")]
#[case(SessionId::new("session-1").as_str().to_string(), "session-1")]
#[case(ObservationId::new("obs-1").as_str().to_string(), "obs-1")]
#[case(OperationId::new("op-1").as_str().to_string(), "op-1")]
#[test]
fn test_id_creation_and_conversions(#[case] actual: String, #[case] expected: &str) {
    assert_eq!(actual, expected);
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
