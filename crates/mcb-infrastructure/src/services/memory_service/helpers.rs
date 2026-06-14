//! Helper methods for `MemoryServiceImpl`.
//!
//! Contains internal implementation methods used by the trait interface.

use mcb_domain::entities::memory::{MemoryFilter, MemorySearchIndex, Observation};
use mcb_domain::error::Result;
use mcb_domain::value_objects::ObservationId;

use super::MemoryServiceImpl;

impl MemoryServiceImpl {
    /// Get timeline of observations around an anchor.
    pub(crate) async fn get_timeline_impl(
        &self,
        org_id: &str,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        self.repository
            .get_timeline(org_id, anchor_id, before, after, filter)
            .await
    }

    /// Get observations by their IDs.
    pub(crate) async fn get_observations_by_ids_impl(
        &self,
        org_id: &str,
        ids: &[ObservationId],
    ) -> Result<Vec<Observation>> {
        self.repository.get_observations_by_ids(org_id, ids).await
    }

    /// Search memories and return indexed results.
    pub(crate) async fn memory_search_impl(
        &self,
        org_id: &str,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        let results = self
            .search_memories_impl(org_id, query, filter, limit)
            .await?;
        Ok(Self::build_memory_index(results))
    }
}
