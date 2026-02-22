//!
//! Domain surface for infrastructure: plug points that infra wires at startup.
//!
//! No dependency on infra crates; infra depends on domain and registers here.

/// Logging facade: set_log_fn + log_operation (macros dispatch here).
pub mod logging;
