//! Unit tests for `HookProcessor` (hooks/processor.rs).

use mcb_server::hooks::processor::HookProcessor;
use rstest::rstest;

#[rstest]
#[test]
fn test_hook_processor_constructs_with_none_memory_service() {
    let processor = HookProcessor::new(None);
    assert!(processor.is_ready());
}
