//! Integration tests — `cargo test -p mcb --test integration`

mod mcp_commands;
#[path = "process_lock_integration.rs"]
mod process_lock;
mod startup_smoke_integration;
mod stdio_transport_integration;
