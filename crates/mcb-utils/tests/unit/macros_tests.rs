//! Tests for constants generated through crate-internal macros.

use mcb_utils::constants::keys::{CREATED_AT, ID};

#[test]
fn define_str_consts_generates_public_string_constants() {
    assert_eq!(ID, "id");
    assert_eq!(CREATED_AT, "created_at");
}
