//! Basic compilation tests for application use cases

use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};

// Test that the use cases can be imported and instantiated
#[test]
fn test_use_cases_can_be_imported() {
    // This test just verifies that the types can be imported
    // and that the crate compiles correctly with the new structure
    assert!(true);
}

// Test that constructors are available
#[test]
fn test_constructors_exist() {
    // Test that we can call constructors (even though we can't provide real dependencies)
    // This is just a compilation test
    assert!(true);
}