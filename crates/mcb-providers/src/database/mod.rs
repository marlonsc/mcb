//! Database adapters.
//!
//! Hosts the `SeaORM`-backed persistence layer and its migrations.

pub mod seaorm;

pub use seaorm::migration;
