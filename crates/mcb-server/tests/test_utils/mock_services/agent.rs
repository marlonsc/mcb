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

use crate::test_utils::helpers::{arc_mutex_hashmap, arc_mutex_vec};

pub struct TestAgentSessionService {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl TestAgentSessionService {
    pub fn new() -> Self {
        Self {
            sessions: arc_mutex_hashmap(),
        }
    }
}

#[async_trait]
impl AgentSessionServiceInterface for TestAgentSessionService {
    async fn create_session(&self, session: AgentSession) -> Result<String> {
        self.sessions
            .lock()
            .expect("Lock poisoned")
            .insert(session.id.clone(), session.clone());
        Ok(session.id.clone())
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>> {
        let sessions = self.sessions.lock().expect("Lock poisoned");
        Ok(sessions.get(session_id).cloned())
    }

    async fn update_session(&self, session: AgentSession) -> Result<()> {
        self.sessions
            .lock()
            .expect("Lock poisoned")
            .insert(session.id.clone(), session);
        Ok(())
    }

    async fn list_sessions(&self, _query: AgentSessionQuery) -> Result<Vec<AgentSession>> {
        let sessions = self.sessions.lock().expect("Lock poisoned");
        Ok(sessions.values().cloned().collect())
    }

    async fn list_sessions_by_project(&self, _project_id: &str) -> Result<Vec<AgentSession>> {
        Ok(vec![])
    }

    async fn list_sessions_by_worktree(&self, _worktree_id: &str) -> Result<Vec<AgentSession>> {
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

pub struct TestAgentRepository {
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    delegations: Arc<Mutex<Vec<Delegation>>>,
    tool_calls: Arc<Mutex<Vec<ToolCall>>>,
    checkpoints: Arc<Mutex<HashMap<String, Checkpoint>>>,
}

impl TestAgentRepository {
    pub fn new() -> Self {
        Self {
            sessions: arc_mutex_hashmap(),
            delegations: arc_mutex_vec(),
            tool_calls: arc_mutex_vec(),
            checkpoints: arc_mutex_hashmap(),
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

        if let Some(project_id) = &query.project_id
            && session.project_id.as_ref() != Some(project_id)
        {
            return false;
        }

        if let Some(worktree_id) = &query.worktree_id
            && session.worktree_id.as_ref() != Some(worktree_id)
        {
            return false;
        }

        true
    }
}

#[async_trait]
impl AgentRepository for TestAgentRepository {
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

    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>> {
        self.list_sessions(AgentSessionQuery {
            project_id: Some(project_id.to_string()),
            ..AgentSessionQuery::default()
        })
        .await
    }

    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>> {
        self.list_sessions(AgentSessionQuery {
            worktree_id: Some(worktree_id.to_string()),
            ..AgentSessionQuery::default()
        })
        .await
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
