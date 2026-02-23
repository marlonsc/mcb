//! Tests for strong-typed ID value objects

use mcb_domain::utils::id;
use mcb_domain::value_objects::ids::{
    ChunkId, CollectionId, ObservationId, OperationId, RepositoryId, SessionId,
};
use rstest::rstest;

#[rstest]
#[case(
    CollectionId::from_uuid(id::deterministic("collection", "test")),
    "collection",
    "test"
)]
#[case(ChunkId::from_uuid(id::deterministic("chunk", "c1")), "chunk", "c1")]
#[case(
    RepositoryId::from_uuid(id::deterministic("repository", "r1")),
    "repository",
    "r1"
)]
#[case(
    SessionId::from_uuid(id::deterministic("session", "s1")),
    "session",
    "s1"
)]
#[case(
    ObservationId::from_uuid(id::deterministic("observation", "o1")),
    "observation",
    "o1"
)]
#[case(
    OperationId::from_uuid(id::deterministic("operation", "op1")),
    "operation",
    "op1"
)]
fn id_string_conversion_determinism<T: ToString>(
    #[case] id_obj: T,
    #[case] namespace: &str,
    #[case] input: &str,
) {
    let expected = id::deterministic(namespace, input).to_string();
    assert_eq!(id_obj.to_string(), expected);
}

#[rstest]
fn collection_id_determinism() {
    let id1 = CollectionId::from_uuid(id::deterministic("collection", "test"));
    let id2 = CollectionId::from_uuid(id::deterministic("collection", "test"));
    assert_eq!(id1, id2);
    assert_eq!(
        id1.to_string(),
        id::deterministic("collection", "test").to_string()
    );
}

#[rstest]
fn collection_id_into_string() {
    let id = CollectionId::from_uuid(id::deterministic("collection", "test"));
    let s: String = id.into();
    assert_eq!(s, id.to_string());
}
