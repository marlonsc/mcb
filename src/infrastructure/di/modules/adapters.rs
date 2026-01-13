//! Adapters DI Module Implementation
//!
//! Contains HTTP client and external provider adapters.

use shaku::module;

use super::traits::AdaptersModule;
use crate::adapters::http_client::HttpClientPool;

module! {
    pub AdaptersModuleImpl: AdaptersModule {
        components = [HttpClientPool],
        providers = []
    }
}
