//! Mock Agent Session Service and Repository implementations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall,
};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::agent_repository::{AgentRepository, AgentSessionQuery};
use mcb_domain::ports::services::AgentSessionServiceInterface;

#[allow(dead_code)]
pub struct MockAgentSessionService {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl MockAgentSessionService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AgentSessionServiceInterface for MockAgentSessionService {
    async fn create_session(&self, session: AgentSession) -> Result<String> {
        Ok(session.id.clone())
    }

    async fn get_session(&self, _session_id: &str) -> Result<Option<AgentSession>> {
        Ok(None)
    }

    async fn update_session(&self, _session: AgentSession) -> Result<()> {
        Ok(())
    }

    async fn list_sessions(&self, _query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        Ok(vec![])
    }

    async fn end_session(
        &self,
        _session_id: &str,
        _status: AgentSessionStatus,
        _result_summary: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    async fn store_delegation(&self, delegation: Delegation) -> Result<String> {
        Ok(delegation.id)
    }

    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String> {
        Ok(tool_call.id)
    }

    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String> {
        Ok(checkpoint.id)
    }

    async fn get_checkpoint(&self, _id: &str) -> Result<Option<Checkpoint>> {
        Ok(None)
    }

    async fn restore_checkpoint(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MockAgentRepository {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    delegations: Arc<Mutex<Vec<Delegation>>>,
    tool_calls: Arc<Mutex<Vec<ToolCall>>>,
    checkpoints: Arc<Mutex<HashMap<String, Checkpoint>>>,
}

#[allow(dead_code)]
impl MockAgentRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            delegations: Arc::new(Mutex::new(Vec::new())),
            tool_calls: Arc::new(Mutex::new(Vec::new())),
            checkpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn matches_query(session: &AgentSession, query: &AgentSessionQuery) -> bool {
        if let Some(summary_id) = &query.session_summary_id
            && session.session_summary_id != *summary_id
        {
            return false;
        }

        if let Some(parent_id) = &query.parent_session_id
            && session.parent_session_id.as_ref() != Some(parent_id)
        {
            return false;
        }

        if let Some(agent_type) = &query.agent_type
            && &session.agent_type != agent_type
        {
            return false;
        }

        if let Some(status) = &query.status
            && &session.status != status
        {
            return false;
        }

        true
    }
}

#[async_trait]
impl AgentRepository for MockAgentRepository {
    async fn create_session(&self, session: &AgentSession) -> Result<()> {
        self.sessions
            .lock()
            .expect("Lock poisoned")
            .insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        let sessions = self.sessions.lock().expect("Lock poisoned");
        Ok(sessions.get(id).cloned())
    }

    async fn update_session(&self, session: &AgentSession) -> Result<()> {
        self.sessions
            .lock()
            .expect("Lock poisoned")
            .insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let sessions = self.sessions.lock().expect("Lock poisoned");
        let mut all: Vec<AgentSession> = sessions.values().cloned().collect();
        all.retain(|session| Self::matches_query(session, &query));

        if let Some(limit) = query.limit {
            all.truncate(limit);
        }

        Ok(all)
    }

    async fn store_delegation(&self, delegation: &Delegation) -> Result<()> {
        self.delegations
            .lock()
            .expect("Lock poisoned")
            .push(delegation.clone());
        Ok(())
    }

    async fn store_tool_call(&self, tool_call: &ToolCall) -> Result<()> {
        self.tool_calls
            .lock()
            .expect("Lock poisoned")
            .push(tool_call.clone());
        Ok(())
    }

    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.checkpoints
            .lock()
            .expect("Lock poisoned")
            .insert(checkpoint.id.clone(), checkpoint.clone());
        Ok(())
    }

    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let checkpoints = self.checkpoints.lock().expect("Lock poisoned");
        Ok(checkpoints.get(id).cloned())
    }

    async fn update_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.checkpoints
            .lock()
            .expect("Lock poisoned")
            .insert(checkpoint.id.clone(), checkpoint.clone());
        Ok(())
    }
}
