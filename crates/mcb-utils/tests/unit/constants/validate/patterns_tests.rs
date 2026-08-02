//! Unit tests for validation pattern constants.

use mcb_utils::constants::validate::{
    CFG_TEST_MARKER, COMMENT_PREFIX, FN_PREFIX, TEST_DIR_FRAGMENT,
};

#[test]
fn validation_marker_constants_match_rust_syntax() {
    assert_eq!(CFG_TEST_MARKER, "#[cfg(test)]");
    assert_eq!(COMMENT_PREFIX, "//");
    assert_eq!(FN_PREFIX, "fn ");
    assert_eq!(TEST_DIR_FRAGMENT, "/tests/");
}
