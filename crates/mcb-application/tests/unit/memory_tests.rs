//! Tests for memory service port (REF003: dedicated test file).
use mcb_domain::ports::services::MemoryServiceInterface;

#[test]
fn test_memory_service_interface_is_object_safe() {
    fn assert_object_safe<T: ?Sized + MemoryServiceInterface>() {}
    assert_object_safe::<dyn MemoryServiceInterface>();
    assert!(true, "MemoryServiceInterface is object safe");
}
