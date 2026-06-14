//! Weaviate Vector Store Provider
//!
//! Implements the `VectorStoreProvider` using Weaviate's REST + GraphQL API.
//!
//! Weaviate has no official Rust SDK; this provider communicates via the
//! reqwest HTTP client. Collections map to Weaviate classes (vectorizer
//! `none`, app-supplied vectors). Vector search uses GraphQL
//! `Get { Class(nearVector: ...) }`; object CRUD uses the REST `/v1/objects`
//! and `/v1/batch/objects` endpoints.

mod admin;
mod browser;
mod client;
mod provider;
mod registry;

pub use client::WeaviateVectorStoreProvider;
pub use registry::weaviate_factory;
