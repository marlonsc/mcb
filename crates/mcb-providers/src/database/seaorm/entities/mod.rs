//! SeaORM entity definitions for all MCB schema tables.
//!
//! Generated from the canonical domain schema in `mcb-domain::schema`.
//! Each module corresponds to one database table and contains:
//! - `Model` struct with `DeriveEntityModel`
//! - `Relation` enum with foreign key definitions
//! - `ActiveModelBehavior` implementation
#![allow(missing_docs)]

pub mod prelude;

pub mod agent_session;
pub mod agent_worktree_assignment;
pub mod api_key;
pub mod branch;
pub mod checkpoint;
pub mod collection;
pub mod delegation;
pub mod error_pattern;
pub mod error_pattern_match;
pub mod file_hash;
pub mod issue_comment;
pub mod issue_label;
pub mod issue_label_assignment;
pub mod observation;
pub mod organization;
pub mod plan;
pub mod plan_review;
pub mod plan_version;
pub mod project;
pub mod project_issue;
pub mod repository;
pub mod session_summary;
pub mod team;
pub mod team_member;
pub mod tool_call;
pub mod user;
pub mod worktree;
