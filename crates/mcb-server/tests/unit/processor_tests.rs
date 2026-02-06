//! Unit tests for HookProcessor (hooks/processor.rs).

use mcb_server::hooks::processor::HookProcessor;

#[test]
fn hook_processor_constructs_with_none_memory_service() {
    let _processor = HookProcessor::new(None);
}
