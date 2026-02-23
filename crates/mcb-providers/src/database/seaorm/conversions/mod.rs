//! Domain ↔ SeaORM entity conversion layer.
//!
//! This module implements the boundary between Clean Architecture domain entities
//! and SeaORM persistence models. Each sub-module provides:
//!
//! - `From<seaorm::Model>` for `domain::Entity` (read path)
//! - `From<domain::Entity>` for `seaorm::ActiveModel` (write path)
//!
//! ## Type Mapping Conventions
//!
//! | Domain Type | SeaORM Type | Notes |
//! |-------------|-------------|-------|
//! | `String` (id) | `String` | UUIDs stored as strings |
//! | Enum (strum) | `String` | `.to_string()` / `.parse()` |
//! | `bool` | `i64` | 0/1 integer encoding |
//! | `Vec<String>` | `String` (JSON) | `serde_json` serialization |
//! | `serde_json::Value` | `String` | JSON string encoding |
//! | `Option<T>` | `Option<T>` | `ActiveValue::Set` / `ActiveValue::NotSet` |

pub mod agent_session;
pub mod agent_worktree_assignment;
pub mod api_key;
pub mod branch;
pub mod checkpoint;
pub mod delegation;
pub mod error_pattern;
pub mod error_pattern_match;
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
