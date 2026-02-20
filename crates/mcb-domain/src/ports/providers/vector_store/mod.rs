use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};

mod admin;
mod browser;
mod provider;

pub use admin::VectorStoreAdmin;
pub use browser::VectorStoreBrowser;
pub use provider::VectorStoreProvider;

pub type MetadataMap = HashMap<String, Value>;
pub type PortResult<T> = Result<T>;
pub type StoreCollectionId = CollectionId;
pub type StoreCollectionInfo = CollectionInfo;
pub type StoreEmbedding = Embedding;
pub type StoreFileInfo = FileInfo;
pub type StoreSearchResult = SearchResult;
