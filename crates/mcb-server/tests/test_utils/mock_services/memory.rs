//! Mock Memory Repository and Service implementations

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::entities::memory::{
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationType,
    SessionSummary,
};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::ports::repositories::memory_repository::FtsSearchResult;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::{Embedding, ObservationId, SessionId};

pub struct MockMemoryRepository {
    pub observations: Arc<Mutex<Vec<Observation>>>,
    pub summaries: Arc<Mutex<Vec<SessionSummary>>>,
}

impl MockMemoryRepository {
    pub fn new() -> Self {
        Self {
            observations: Arc::new(Mutex::new(Vec::new())),
            summaries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryRepository for MockMemoryRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        self.observations
            .lock()
            .expect("Lock poisoned")
            .push(observation.clone());
        Ok(())
    }

    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs.iter().find(|o| o.id == id.as_str()).cloned())
    }

    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs.iter().find(|o| o.content_hash == content_hash).cloned())
    }

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        self.summaries
            .lock()
            .expect("Lock poisoned")
            .push(summary.clone());
        Ok(())
    }

    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        let summaries = self.summaries.lock().expect("Lock poisoned");
        Ok(summaries
            .iter()
            .find(|s| s.session_id == session_id.as_str())
            .cloned())
    }

    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        let anchor_idx = obs.iter().position(|o| o.id == anchor_id.as_str());

        match anchor_idx {
            None => Ok(vec![]),
            Some(idx) => {
                let start = idx.saturating_sub(before);
                let end = std::cmp::min(idx + after + 1, obs.len());

                let timeline: Vec<Observation> = obs[start..end]
                    .iter()
                    .filter(|o| {
                        if let Some(ref f) = filter
                            && let Some(ref session) = f.session_id
                            && o.metadata.session_id.as_ref() != Some(session)
                        {
                            return false;
                        }
                        true
                    })
                    .cloned()
                    .collect();

                Ok(timeline)
            }
        }
    }

    async fn search(&self, _query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        Ok(obs
            .iter()
            .take(limit)
            .map(|o| FtsSearchResult {
                id: o.id.clone(),
                rank: 1.0,
            })
            .collect())
    }

    async fn delete_observation(&self, id: &ObservationId) -> Result<()> {
        let mut obs = self.observations.lock().expect("Lock poisoned");
        obs.retain(|o| o.id != id.as_str());
        Ok(())
    }

    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        let obs = self.observations.lock().expect("Lock poisoned");
        let id_strs: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
        Ok(obs
            .iter()
            .filter(|o| id_strs.contains(&o.id.as_str()))
            .cloned()
            .collect())
    }
}

pub struct MockMemoryService {
    pub should_fail: Arc<AtomicBool>,
    pub error_message: Arc<Mutex<String>>,
    pub observations: Arc<Mutex<Vec<Observation>>>,
    pub summaries: Arc<Mutex<Vec<SessionSummary>>>,
}

impl MockMemoryService {
    pub fn new() -> Self {
        Self {
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated memory failure".to_string())),
            observations: Arc::new(Mutex::new(Vec::new())),
            summaries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMemoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryServiceInterface for MockMemoryService {
    async fn store_observation(
        &self,
        content: String,
        observation_type: ObservationType,
        tags: Vec<String>,
        metadata: mcb_domain::entities::memory::ObservationMetadata,
    ) -> Result<(ObservationId, bool)> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let obs = Observation {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "mock-project".to_string(),
            content,
            content_hash: "mock-hash".to_string(),
            tags,
            observation_type,
            metadata,
            created_at: chrono::Utc::now().timestamp(),
            embedding_id: None,
        };
        let id = obs.id.clone();
        self.observations.lock().expect("Lock poisoned").push(obs);
        Ok((ObservationId::new(id), false))
    }

    async fn search_memories(
        &self,
        _query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<MemorySearchResult> = observations
            .iter()
            .filter(|obs| {
                if let Some(ref f) = filter
                    && let Some(ref session) = f.session_id
                    && obs.metadata.session_id.as_ref() != Some(session)
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|obs| MemorySearchResult {
                id: obs.id.clone(),
                observation: obs.clone(),
                similarity_score: 0.95,
            })
            .collect();
        Ok(results)
    }

    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let summaries = self.summaries.lock().expect("Lock poisoned");
        Ok(summaries
            .iter()
            .find(|s| s.session_id == session_id.as_str())
            .cloned())
    }

    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        let observations = self.observations.lock().expect("Lock poisoned");
        Ok(observations.iter().find(|o| o.id == id.as_str()).cloned())
    }

    async fn embed_content(&self, _content: &str) -> Result<Embedding> {
        Ok(Embedding {
            vector: vec![0.0; 384],
            model: "mock-model".to_string(),
            dimensions: 384,
        })
    }

    async fn memory_search(
        &self,
        _query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<MemorySearchIndex> = observations
            .iter()
            .filter(|obs| {
                if let Some(ref f) = filter
                    && let Some(ref session) = f.session_id
                    && obs.metadata.session_id.as_ref() != Some(session)
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|obs| MemorySearchIndex {
                id: obs.id.clone(),
                observation_type: format!("{:?}", obs.observation_type),
                relevance_score: 0.9,
                tags: obs.tags.clone(),
                content_preview: obs
                    .content
                    .chars()
                    .take(120)
                    .chain(std::iter::once('…'))
                    .collect::<String>(),
                session_id: obs.metadata.session_id.clone(),
                repo_id: None,
                file_path: None,
                created_at: obs.created_at,
            })
            .collect();
        Ok(results)
    }

    async fn create_session_summary(
        &self,
        session_id: SessionId,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let summary = SessionSummary {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: "mock-project".to_string(),
            session_id: session_id.into_string(),
            topics,
            decisions,
            next_steps,
            key_files,
            created_at: chrono::Utc::now().timestamp(),
        };
        let id = summary.id.clone();
        self.summaries.lock().expect("Lock poisoned").push(summary);
        Ok(id)
    }

    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let anchor_idx = observations.iter().position(|o| o.id == anchor_id.as_str());

        match anchor_idx {
            None => Ok(vec![]),
            Some(idx) => {
                let start = idx.saturating_sub(before);
                let end = std::cmp::min(idx + after + 1, observations.len());

                let timeline: Vec<Observation> = observations[start..end]
                    .iter()
                    .filter(|obs| {
                        if let Some(ref f) = filter
                            && let Some(ref session) = f.session_id
                            && obs.metadata.session_id.as_ref() != Some(session)
                        {
                            return false;
                        }
                        true
                    })
                    .cloned()
                    .collect();

                Ok(timeline)
            }
        }
    }

    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let observations = self.observations.lock().expect("Lock poisoned");
        let results: Vec<Observation> = observations
            .iter()
            .filter(|obs| ids.iter().any(|id| id.as_str() == obs.id))
            .cloned()
            .collect();

        Ok(results)
    }
}
