//! Pinecone Vector Store Provider
//!
//! Implements the `VectorStoreProvider` using Pinecone's cloud vector database REST API.
//!
//! Pinecone is a managed vector database optimized for machine learning applications.
//! This provider communicates via Pinecone's REST API using the reqwest HTTP client.

mod admin;
mod browser;
mod client;
mod provider;
mod registry;

pub use client::PineconeVectorStoreProvider;
pub use registry::pinecone_factory;
