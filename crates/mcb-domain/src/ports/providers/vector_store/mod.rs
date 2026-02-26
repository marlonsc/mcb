//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!

mod admin;
mod browser;
mod provider;

pub use admin::VectorStoreAdmin;
pub use browser::VectorStoreBrowser;
pub use provider::VectorStoreProvider;
