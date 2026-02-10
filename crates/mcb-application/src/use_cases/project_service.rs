//! Project Service Implementation

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, Project, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::project::ProjectServiceInterface;
use uuid::Uuid;

/// Service implementation for managing project workflow resources.
pub struct ProjectServiceImpl {
    repository: Arc<dyn ProjectRepository>,
}

impl ProjectServiceImpl {
    /// Creates a new ProjectServiceImpl.
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

#[async_trait]
impl ProjectServiceInterface for ProjectServiceImpl {
    // Project operations
    async fn get_project(&self, id: &str) -> Result<Project> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Project {}", id)))
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        self.repository.list().await
    }

    // Phase operations
    async fn create_phase(
        &self,
        project_id: &str,
        name: String,
        description: String,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        // Determine sequence - append to end
        let existing = self.repository.list_phases(project_id).await?;
        let sequence = existing.len() as i32 + 1;

        let phase = ProjectPhase {
            id: id.clone(),
            project_id: project_id.to_string(),
            name,
            description,
            sequence,
            status: PhaseStatus::Planned,
            started_at: None,
            completed_at: None,
            created_at: Self::current_timestamp(),
            updated_at: Self::current_timestamp(),
        };

        self.repository.create_phase(&phase).await?;
        Ok(id)
    }

    async fn update_phase(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<PhaseStatus>,
    ) -> Result<()> {
        let mut phase = self
            .repository
            .get_phase(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Phase {}", id)))?;

        if let Some(n) = name {
            phase.name = n;
        }
        if let Some(d) = description {
            phase.description = d;
        }
        if let Some(s) = status {
            if s == PhaseStatus::InProgress && phase.status != PhaseStatus::InProgress {
                phase.started_at = Some(Self::current_timestamp());
            } else if s == PhaseStatus::Completed && phase.status != PhaseStatus::Completed {
                phase.completed_at = Some(Self::current_timestamp());
            }
            phase.status = s;
        }
        phase.updated_at = Self::current_timestamp();

        self.repository.update_phase(&phase).await
    }

    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
        self.repository.list_phases(project_id).await
    }

    async fn delete_phase(&self, id: &str) -> Result<()> {
        self.repository.delete_phase(id).await
    }

    // Issue operations
    async fn create_issue(
        &self,
        project_id: &str,
        title: String,
        description: String,
        issue_type: IssueType,
        priority: i32,
        phase_id: Option<String>,
        assignee: Option<String>,
        labels: Vec<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let issue = ProjectIssue {
            id: id.clone(),
            project_id: project_id.to_string(),
            phase_id,
            title,
            description,
            issue_type,
            status: IssueStatus::Open,
            priority,
            assignee,
            labels,
            created_at: Self::current_timestamp(),
            updated_at: Self::current_timestamp(),
            closed_at: None,
        };

        self.repository.create_issue(&issue).await?;
        Ok(id)
    }

    async fn update_issue(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<IssueStatus>,
        priority: Option<i32>,
        assignee: Option<String>,
        labels: Option<Vec<String>>,
    ) -> Result<()> {
        let mut issue = self
            .repository
            .get_issue(id)
            .await?
            .ok_or_else(|| Error::not_found(format!("Issue {}", id)))?;

        if let Some(t) = title {
            issue.title = t;
        }
        if let Some(d) = description {
            issue.description = d;
        }
        if let Some(s) = status {
            if (s == IssueStatus::Resolved || s == IssueStatus::Closed)
                && (issue.status != IssueStatus::Resolved && issue.status != IssueStatus::Closed)
            {
                issue.closed_at = Some(Self::current_timestamp());
            } else if s != IssueStatus::Resolved && s != IssueStatus::Closed {
                issue.closed_at = None;
            }
            issue.status = s;
        }
        if let Some(p) = priority {
            issue.priority = p;
        }
        if let Some(a) = assignee {
            issue.assignee = if a.is_empty() { None } else { Some(a) };
        }
        if let Some(l) = labels {
            issue.labels = l;
        }
        issue.updated_at = Self::current_timestamp();

        self.repository.update_issue(&issue).await
    }

    async fn list_issues(
        &self,
        project_id: &str,
        filter: Option<IssueFilter>,
    ) -> Result<Vec<ProjectIssue>> {
        if let Some(mut f) = filter {
            f.project_id = Some(project_id.to_string());
            self.repository.filter_issues(&f).await
        } else {
            self.repository.list_issues(project_id).await
        }
    }

    async fn delete_issue(&self, id: &str) -> Result<()> {
        self.repository.delete_issue(id).await
    }

    // Dependency operations
    async fn add_dependency(
        &self,
        from_issue_id: String,
        to_issue_id: String,
        dependency_type: DependencyType,
    ) -> Result<String> {
        // Validate issues exist
        if self.repository.get_issue(&from_issue_id).await?.is_none() {
            return Err(Error::not_found(format!("Issue {}", from_issue_id)));
        }
        if self.repository.get_issue(&to_issue_id).await?.is_none() {
            return Err(Error::not_found(format!("Issue {}", to_issue_id)));
        }

        let id = Uuid::new_v4().to_string();
        let dep = ProjectDependency {
            id: id.clone(),
            from_issue_id,
            to_issue_id,
            dependency_type,
            created_at: Self::current_timestamp(),
        };

        self.repository.add_dependency(&dep).await?;
        Ok(id)
    }

    async fn remove_dependency(&self, id: &str) -> Result<()> {
        self.repository.remove_dependency(id).await
    }

    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>> {
        self.repository.list_dependencies(project_id).await
    }

    // Decision operations
    async fn create_decision(
        &self,
        project_id: &str,
        title: String,
        context: String,
        decision: String,
        consequences: String,
        issue_id: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let decision_record = ProjectDecision {
            id: id.clone(),
            project_id: project_id.to_string(),
            issue_id,
            title,
            context,
            decision,
            consequences,
            created_at: Self::current_timestamp(),
        };

        self.repository.create_decision(&decision_record).await?;
        Ok(id)
    }

    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
        self.repository.list_decisions(project_id).await
    }

    async fn delete_decision(&self, id: &str) -> Result<()> {
        self.repository.delete_decision(id).await
    }
}
