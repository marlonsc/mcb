//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Re-exports metadata constants from domain layer.

pub use mcb_domain::constants::keys::{
    METADATA_KEY_CHUNK_TYPE, METADATA_KEY_END_LINE, METADATA_KEY_FILE_PATH,
    METADATA_KEY_START_LINE, METADATA_KEY_VECTORS_COUNT,
};
