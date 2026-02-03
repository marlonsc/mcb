//! Tests for memory service port (REF003: dedicated test file).

use mcb_application::ports::MemoryServiceInterface;

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_memory_service_interface_is_object_safe() {
    fn _assert_object_safe(_: &dyn MemoryServiceInterface) {}
    assert!(true, "MemoryServiceInterface is object-safe");
}
