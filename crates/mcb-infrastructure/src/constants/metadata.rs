//! Re-exports metadata constants from domain layer.
//!
//! These constants were moved to mcb-domain to fix layering.
//! Infrastructure should import domain constants, not define its own.

pub use mcb_domain::constants::keys::{
    METADATA_KEY_CHUNK_TYPE, METADATA_KEY_END_LINE, METADATA_KEY_FILE_PATH,
    METADATA_KEY_START_LINE, METADATA_KEY_VECTORS_COUNT,
};
