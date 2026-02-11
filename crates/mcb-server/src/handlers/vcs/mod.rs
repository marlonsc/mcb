//! VCS handler for version control operations.
//!
//! This module provides a unified handler for VCS-related MCP tool operations.

mod analyze_impact;
mod compare_branches;
mod handler;
mod index_repo;
mod list_repos;
mod responses;
mod search_branch;

pub use handler::VcsHandler;
