//! MCB — Memory Context Browser binary crate.
pub mod cli;

/// Re-export of the domain layer.
pub mod domain {
    pub use mcb_domain::*;
}

/// Re-export of the server layer.
pub mod server {
    pub use mcb_server::*;
}

/// Re-export of the infrastructure layer.
pub mod infrastructure {
    pub use mcb_infrastructure::*;
}

pub use domain::*;
pub use server::McbApp;
pub use server::{McpServer, McpServerBuilder};
