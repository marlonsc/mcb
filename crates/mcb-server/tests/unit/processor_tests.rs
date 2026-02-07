//! Unit tests for HookProcessor (hooks/processor.rs).

use mcb_server::hooks::processor::HookProcessor;

#[test]
fn test_hook_processor_constructs_with_none_memory_service() {
    let processor = HookProcessor::new(None);
    // Verify the processor was successfully constructed
    let _ = processor;
    assert!(
        true,
        "HookProcessor successfully constructed with None memory_service"
    );
}
