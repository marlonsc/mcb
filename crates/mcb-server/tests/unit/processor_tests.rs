//! Unit tests for HookProcessor (hooks/processor.rs).

use mcb_server::hooks::processor::HookProcessor;

#[test]
fn test_hook_processor_constructs_with_none_memory_service() {
    let _processor = HookProcessor::new(None);
    // Success if it reaches here without panic
    assert!(true);
}
