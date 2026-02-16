//! Pending-task and stub detection labels.
//!
//! Labels are built via `concat!()` to prevent the source file itself from
//! triggering ripgrep-based lint rules for task-comment patterns.

/// Label for pending task comments (first priority).
pub const PENDING_LABEL_TODO: &str = concat!("TO", "DO");

/// Label for fix-needed comments.
pub const PENDING_LABEL_FIXME: &str = concat!("FI", "XME");

/// Label for attention-needed comments.
pub const PENDING_LABEL_XXX: &str = concat!("X", "XX");

/// Label for workaround/shortcut comments.
pub const PENDING_LABEL_HACK: &str = concat!("HA", "CK");

/// Label for panic-stub detection (unimplemented placeholders).
pub const STUB_PANIC_LABEL: &str = concat!("panic(", "TO", "DO)");

/// Label used in reporter tests (identical to pending-task label).
pub const REPORT_TEST_PENDING_LABEL: &str = concat!("TO", "DO");
