//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Re-exports naming constants from the central constants module.
//!
//! All CA and module naming constants live in [`crate::constants::ca`].

pub use crate::constants::ca::{
    CA_ADAPTERS_DIR, CA_ADAPTERS_REPOSITORY_DIR, CA_DI_DIR, CA_DOMAIN_PROVIDER_KEYWORD,
    CA_DOMAIN_REPOSITORY_KEYWORD, CA_HANDLER_DIRS, CA_HANDLER_KEYWORD, CA_INFRA_ADAPTER_KEYWORD,
    CA_INFRA_IMPL_SUFFIX, CA_MODULE_KEYWORD, CA_PORTS_DIR, CA_PORTS_PROVIDERS_DIR,
    CA_REPOSITORIES_DIR, MODULE_FILE_NAME, MODULE_SPECIAL_FILES,
};
