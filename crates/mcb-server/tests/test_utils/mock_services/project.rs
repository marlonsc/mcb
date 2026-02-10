//! Mock implementations of project services
use async_trait::async_trait;
use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::{ProjectDetectorService, ProjectServiceInterface};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct MockProjectService;

impl MockProjectService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectDetectorService for MockProjectService {
    async fn detect_all(&self, _path: &Path) -> Vec<ProjectType> {
        vec![]
    }
}

#[async_trait]
impl ProjectServiceInterface for MockProjectService {
    async fn get_project(
        &self,
        _id: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Ok(mcb_domain::entities::project::Project {
            id: "test".to_string(),
            name: "test".to_string(),
            path: "/tmp/test".to_string(),
            created_at: 0,
            updated_at: 0,
        })
    }

    async fn list_projects(
        &self,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }

    async fn create_phase(
        &self,
        _project_id: &str,
        _name: String,
        _description: String,
    ) -> mcb_domain::error::Result<String> {
        unimplemented!()
    }
    async fn update_phase(
        &self,
        _id: &str,
        _name: Option<String>,
        _description: Option<String>,
        _status: Option<PhaseStatus>,
    ) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
    async fn list_phases(&self, _project_id: &str) -> mcb_domain::error::Result<Vec<ProjectPhase>> {
        unimplemented!()
    }
    async fn delete_phase(&self, _id: &str) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
    async fn create_issue(
        &self,
        _project_id: &str,
        _title: String,
        _description: String,
        _issue_type: IssueType,
        _priority: i32,
        _phase_id: Option<String>,
        _assignee: Option<String>,
        _labels: Vec<String>,
    ) -> mcb_domain::error::Result<String> {
        unimplemented!()
    }
    async fn update_issue(
        &self,
        _id: &str,
        _title: Option<String>,
        _description: Option<String>,
        _status: Option<IssueStatus>,
        _priority: Option<i32>,
        _assignee: Option<String>,
        _labels: Option<Vec<String>>,
    ) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
    async fn list_issues(
        &self,
        _project_id: &str,
        _filter: Option<IssueFilter>,
    ) -> mcb_domain::error::Result<Vec<ProjectIssue>> {
        unimplemented!()
    }
    async fn delete_issue(&self, _id: &str) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
    async fn add_dependency(
        &self,
        _from_issue_id: String,
        _to_issue_id: String,
        _dependency_type: DependencyType,
    ) -> mcb_domain::error::Result<String> {
        unimplemented!()
    }
    async fn remove_dependency(&self, _id: &str) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
    async fn list_dependencies(
        &self,
        _project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDependency>> {
        unimplemented!()
    }
    async fn create_decision(
        &self,
        _project_id: &str,
        _title: String,
        _context: String,
        _decision: String,
        _consequences: String,
        _issue_id: Option<String>,
    ) -> mcb_domain::error::Result<String> {
        unimplemented!()
    }
    async fn list_decisions(
        &self,
        _project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDecision>> {
        unimplemented!()
    }
    async fn delete_decision(&self, _id: &str) -> mcb_domain::error::Result<()> {
        unimplemented!()
    }
}

#[allow(dead_code)]
pub struct MockProjectRepository;

#[allow(dead_code)]
impl MockProjectRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn create(
        &self,
        _project: &mcb_domain::entities::project::Project,
    ) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn get_by_id(
        &self,
        _id: &str,
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
    }
    async fn get_by_name(
        &self,
        _name: &str,
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
    }
    async fn get_by_path(
        &self,
        _path: &str,
    ) -> mcb_domain::error::Result<Option<mcb_domain::entities::project::Project>> {
        Ok(None)
    }
    async fn list(&self) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }
    async fn update(
        &self,
        _project: &mcb_domain::entities::project::Project,
    ) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn delete(&self, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn create_phase(&self, _phase: &ProjectPhase) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn get_phase(&self, _id: &str) -> mcb_domain::error::Result<Option<ProjectPhase>> {
        Ok(None)
    }
    async fn update_phase(&self, _phase: &ProjectPhase) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn list_phases(&self, _project_id: &str) -> mcb_domain::error::Result<Vec<ProjectPhase>> {
        Ok(vec![])
    }
    async fn delete_phase(&self, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn create_issue(&self, _issue: &ProjectIssue) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn get_issue(&self, _id: &str) -> mcb_domain::error::Result<Option<ProjectIssue>> {
        Ok(None)
    }
    async fn update_issue(&self, _issue: &ProjectIssue) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn list_issues(&self, _project_id: &str) -> mcb_domain::error::Result<Vec<ProjectIssue>> {
        Ok(vec![])
    }
    async fn filter_issues(
        &self,
        _filter: &IssueFilter,
    ) -> mcb_domain::error::Result<Vec<ProjectIssue>> {
        Ok(vec![])
    }
    async fn delete_issue(&self, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn add_dependency(&self, _dep: &ProjectDependency) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn remove_dependency(&self, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn list_dependencies(
        &self,
        _project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDependency>> {
        Ok(vec![])
    }
    async fn create_decision(&self, _decision: &ProjectDecision) -> mcb_domain::error::Result<()> {
        Ok(())
    }
    async fn get_decision(&self, _id: &str) -> mcb_domain::error::Result<Option<ProjectDecision>> {
        Ok(None)
    }
    async fn list_decisions(
        &self,
        _project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDecision>> {
        Ok(vec![])
    }
    async fn delete_decision(&self, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }
}

pub struct MockProjectWorkflowService {
    _phases: Mutex<HashMap<String, ProjectPhase>>,
    _issues: Mutex<HashMap<String, ProjectIssue>>,
    _dependencies: Mutex<HashMap<String, ProjectDependency>>,
    _decisions: Mutex<HashMap<String, ProjectDecision>>,
    _phase_counter: Mutex<i32>,
}

#[allow(dead_code)]
impl MockProjectWorkflowService {
    pub fn new() -> Self {
        Self {
            _phases: Mutex::new(HashMap::new()),
            _issues: Mutex::new(HashMap::new()),
            _dependencies: Mutex::new(HashMap::new()),
            _decisions: Mutex::new(HashMap::new()),
            _phase_counter: Mutex::new(0),
        }
    }
}

impl Default for MockProjectWorkflowService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProjectServiceInterface for MockProjectWorkflowService {
    async fn get_project(
        &self,
        _id: &str,
    ) -> mcb_domain::error::Result<mcb_domain::entities::project::Project> {
        Err(mcb_domain::error::Error::not_found("Project not found"))
    }

    async fn list_projects(
        &self,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::entities::project::Project>> {
        Ok(vec![])
    }

    async fn create_phase(
        &self,
        project_id: &str,
        name: String,
        description: String,
    ) -> mcb_domain::error::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut counter = self._phase_counter.lock().unwrap();
        *counter += 1;
        let phase = ProjectPhase {
            id: id.clone(),
            project_id: project_id.to_string(),
            name,
            description,
            sequence: *counter,
            status: PhaseStatus::Planned,
            started_at: None,
            completed_at: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        self._phases.lock().unwrap().insert(id.clone(), phase);
        Ok(id)
    }

    async fn update_phase(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<PhaseStatus>,
    ) -> mcb_domain::error::Result<()> {
        let mut phases = self._phases.lock().unwrap();
        if let Some(phase) = phases.get_mut(id) {
            if let Some(n) = name {
                phase.name = n;
            }
            if let Some(d) = description {
                phase.description = d;
            }
            if let Some(s) = status {
                phase.status = s;
            }
            phase.updated_at = chrono::Utc::now().timestamp();
        }
        Ok(())
    }

    async fn list_phases(&self, project_id: &str) -> mcb_domain::error::Result<Vec<ProjectPhase>> {
        let phases = self._phases.lock().unwrap();
        Ok(phases
            .values()
            .filter(|p| p.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn delete_phase(&self, id: &str) -> mcb_domain::error::Result<()> {
        self._phases.lock().unwrap().remove(id);
        Ok(())
    }

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
    ) -> mcb_domain::error::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
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
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            closed_at: None,
        };
        self._issues.lock().unwrap().insert(id.clone(), issue);
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
    ) -> mcb_domain::error::Result<()> {
        let mut issues = self._issues.lock().unwrap();
        if let Some(issue) = issues.get_mut(id) {
            if let Some(t) = title {
                issue.title = t;
            }
            if let Some(d) = description {
                issue.description = d;
            }
            if let Some(s) = status {
                issue.status = s;
            }
            if let Some(p) = priority {
                issue.priority = p;
            }
            if let Some(a) = assignee {
                issue.assignee = Some(a);
            }
            if let Some(l) = labels {
                issue.labels = l;
            }
            issue.updated_at = chrono::Utc::now().timestamp();
        }
        Ok(())
    }

    async fn list_issues(
        &self,
        project_id: &str,
        _filter: Option<IssueFilter>,
    ) -> mcb_domain::error::Result<Vec<ProjectIssue>> {
        let issues = self._issues.lock().unwrap();
        Ok(issues
            .values()
            .filter(|i| i.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn delete_issue(&self, id: &str) -> mcb_domain::error::Result<()> {
        self._issues.lock().unwrap().remove(id);
        Ok(())
    }

    async fn add_dependency(
        &self,
        from_issue_id: String,
        to_issue_id: String,
        dependency_type: DependencyType,
    ) -> mcb_domain::error::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let dep = ProjectDependency {
            id: id.clone(),
            from_issue_id,
            to_issue_id,
            dependency_type,
            created_at: chrono::Utc::now().timestamp(),
        };
        self._dependencies.lock().unwrap().insert(id.clone(), dep);
        Ok(id)
    }

    async fn remove_dependency(&self, id: &str) -> mcb_domain::error::Result<()> {
        self._dependencies.lock().unwrap().remove(id);
        Ok(())
    }

    async fn list_dependencies(
        &self,
        _project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDependency>> {
        Ok(self
            ._dependencies
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }

    async fn create_decision(
        &self,
        project_id: &str,
        title: String,
        context: String,
        decision: String,
        consequences: String,
        issue_id: Option<String>,
    ) -> mcb_domain::error::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let dec = ProjectDecision {
            id: id.clone(),
            project_id: project_id.to_string(),
            issue_id,
            title,
            context,
            decision,
            consequences,
            created_at: chrono::Utc::now().timestamp(),
        };
        self._decisions.lock().unwrap().insert(id.clone(), dec);
        Ok(id)
    }

    async fn list_decisions(
        &self,
        project_id: &str,
    ) -> mcb_domain::error::Result<Vec<ProjectDecision>> {
        let decisions = self._decisions.lock().unwrap();
        Ok(decisions
            .values()
            .filter(|d| d.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn delete_decision(&self, id: &str) -> mcb_domain::error::Result<()> {
        self._decisions.lock().unwrap().remove(id);
        Ok(())
    }
}
