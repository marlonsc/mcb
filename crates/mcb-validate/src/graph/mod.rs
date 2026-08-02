//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Dependency graph construction and analysis.
//!
//! Builds a directed graph from extracted code facts using `petgraph`,
//! enabling cycle detection and layer-boundary validation.

pub mod dep_graph;

pub use dep_graph::DependencyGraph;
