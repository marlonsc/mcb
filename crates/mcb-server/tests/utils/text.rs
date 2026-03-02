//! Text helpers — re-exports from centralized sources.

// Production text helpers from mcb-server
pub use mcb_server::utils::text::{extract_text, extract_text_with_sep};

// JSON parsing helpers from centralized mcb-domain
pub use mcb_domain::test_json_helpers::{parse_count_from_json_text, parse_json, parse_json_text};
