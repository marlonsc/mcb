use std::sync::Arc;

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::ports::services::IssueEntityServiceInterface;
use mcb_domain::value_objects::OrgContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use crate::error_mapping::to_opaque_mcp_error;
use crate::handler_helpers::{ok_json, ok_text, require_id};

/// Handler for the consolidated `issue_entity` MCP tool.
pub struct IssueEntityHandler {
    service: Arc<dyn IssueEntityServiceInterface>,
}

impl IssueEntityHandler {
    /// Create a new handler wrapping the given service.
    pub fn new(service: Arc<dyn IssueEntityServiceInterface>) -> Self {
        Self { service }
    }

    /// Route an incoming `issue_entity` tool call to the appropriate CRUD operation.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IssueEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        // TODO(phase-1): extract org_id from auth token / request context
        let org_ctx = OrgContext::default();
        let org_id = args.org_id.as_deref().unwrap_or(org_ctx.org_id.as_str());

        match (args.action, args.resource) {
            (IssueEntityAction::Create, IssueEntityResource::Issue) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for create", None))?;
                let mut issue: ProjectIssue = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                issue.org_id = org_id.to_string();
                self.service
                    .create_issue(&issue)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&issue)
            }
            (IssueEntityAction::Get, IssueEntityResource::Issue) => {
                let id = require_id(&args.id)?;
                let issue = self
                    .service
                    .get_issue(org_id, &id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&issue)
            }
            (IssueEntityAction::List, IssueEntityResource::Issue) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                let issues = self
                    .service
                    .list_issues(org_id, project_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&issues)
            }
            (IssueEntityAction::Update, IssueEntityResource::Issue) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required for update", None))?;
                let mut issue: ProjectIssue = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                issue.org_id = org_id.to_string();
                self.service
                    .update_issue(&issue)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("updated")
            }
            (IssueEntityAction::Delete, IssueEntityResource::Issue) => {
                let id = require_id(&args.id)?;
                self.service
                    .delete_issue(org_id, &id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::Comment) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let comment: IssueComment = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                self.service
                    .create_comment(&comment)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&comment)
            }
            (IssueEntityAction::Get, IssueEntityResource::Comment) => {
                let id = require_id(&args.id)?;
                let comment = self
                    .service
                    .get_comment(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&comment)
            }
            (IssueEntityAction::List, IssueEntityResource::Comment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                let comments = self
                    .service
                    .list_comments_by_issue(issue_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&comments)
            }
            (IssueEntityAction::Delete, IssueEntityResource::Comment) => {
                let id = require_id(&args.id)?;
                self.service
                    .delete_comment(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::Label) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let mut label: IssueLabel = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                label.org_id = org_id.to_string();
                self.service
                    .create_label(&label)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&label)
            }
            (IssueEntityAction::Get, IssueEntityResource::Label) => {
                let id = require_id(&args.id)?;
                let label = self
                    .service
                    .get_label(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&label)
            }
            (IssueEntityAction::List, IssueEntityResource::Label) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                let labels = self
                    .service
                    .list_labels(org_id, project_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&labels)
            }
            (IssueEntityAction::Delete, IssueEntityResource::Label) => {
                let id = require_id(&args.id)?;
                self.service
                    .delete_label(&id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::LabelAssignment) => {
                let data = args
                    .data
                    .ok_or_else(|| McpError::invalid_params("data required", None))?;
                let assignment: IssueLabelAssignment = serde_json::from_value(data)
                    .map_err(|_| McpError::invalid_params("invalid data", None))?;
                self.service
                    .assign_label(&assignment)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("assigned")
            }
            (IssueEntityAction::Delete, IssueEntityResource::LabelAssignment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                let label_id = args
                    .label_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("label_id required", None))?;
                self.service
                    .unassign_label(issue_id, label_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_text("unassigned")
            }
            (IssueEntityAction::List, IssueEntityResource::LabelAssignment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                let labels = self
                    .service
                    .list_labels_for_issue(issue_id)
                    .await
                    .map_err(to_opaque_mcp_error)?;
                ok_json(&labels)
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Unsupported action {:?} for resource {:?}",
                    args.action, args.resource
                ),
                None,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::error::Result;
    use mcb_domain::keys::DEFAULT_ORG_ID;
    use std::sync::Mutex;

    struct MockIssueEntityService {
        issues: Mutex<Vec<ProjectIssue>>,
    }

    impl MockIssueEntityService {
        fn new() -> Self {
            Self {
                issues: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl IssueEntityServiceInterface for MockIssueEntityService {
        async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
            self.issues.lock().expect("lock issues").push(issue.clone());
            Ok(())
        }
        async fn get_issue(&self, _org_id: &str, id: &str) -> Result<ProjectIssue> {
            self.issues
                .lock()
                .expect("lock issues")
                .iter()
                .find(|i| i.id == id)
                .cloned()
                .ok_or_else(|| mcb_domain::error::Error::not_found(format!("Issue {id}")))
        }
        async fn list_issues(&self, _org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
            Ok(self
                .issues
                .lock()
                .expect("lock issues")
                .iter()
                .filter(|i| i.project_id == project_id)
                .cloned()
                .collect())
        }
        async fn update_issue(&self, _issue: &ProjectIssue) -> Result<()> {
            Ok(())
        }
        async fn delete_issue(&self, _org_id: &str, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_comment(&self, _comment: &IssueComment) -> Result<()> {
            Ok(())
        }
        async fn get_comment(&self, _id: &str) -> Result<IssueComment> {
            Err(mcb_domain::error::Error::not_found("comment"))
        }
        async fn list_comments_by_issue(&self, _issue_id: &str) -> Result<Vec<IssueComment>> {
            Ok(vec![])
        }
        async fn delete_comment(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn create_label(&self, _label: &IssueLabel) -> Result<()> {
            Ok(())
        }
        async fn get_label(&self, _id: &str) -> Result<IssueLabel> {
            Err(mcb_domain::error::Error::not_found("label"))
        }
        async fn list_labels(&self, _org_id: &str, _project_id: &str) -> Result<Vec<IssueLabel>> {
            Ok(vec![])
        }
        async fn delete_label(&self, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn assign_label(&self, _assignment: &IssueLabelAssignment) -> Result<()> {
            Ok(())
        }
        async fn unassign_label(&self, _issue_id: &str, _label_id: &str) -> Result<()> {
            Ok(())
        }
        async fn list_labels_for_issue(&self, _issue_id: &str) -> Result<Vec<IssueLabel>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_list_issues() {
        let service = Arc::new(MockIssueEntityService::new());
        {
            service
                .issues
                .lock()
                .expect("lock issues")
                .push(ProjectIssue {
                    id: "i1".into(),
                    org_id: DEFAULT_ORG_ID.into(),
                    project_id: "proj-1".into(),
                    created_by: "user-1".into(),
                    phase_id: None,
                    title: "title".into(),
                    description: "desc".into(),
                    issue_type: mcb_domain::entities::project::IssueType::Task,
                    status: mcb_domain::entities::project::IssueStatus::Open,
                    priority: 1,
                    assignee: None,
                    labels: vec![],
                    estimated_minutes: None,
                    actual_minutes: None,
                    notes: String::new(),
                    design: String::new(),
                    parent_issue_id: None,
                    created_at: 0,
                    updated_at: 0,
                    closed_at: None,
                    closed_reason: String::new(),
                });
        }
        let handler = IssueEntityHandler::new(service);
        let args = IssueEntityArgs {
            action: IssueEntityAction::List,
            resource: IssueEntityResource::Issue,
            id: None,
            org_id: None,
            project_id: Some("proj-1".into()),
            issue_id: None,
            label_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_issue() {
        let service = Arc::new(MockIssueEntityService::new());
        {
            service
                .issues
                .lock()
                .expect("lock issues")
                .push(ProjectIssue {
                    id: "i1".into(),
                    org_id: DEFAULT_ORG_ID.into(),
                    project_id: "proj-1".into(),
                    created_by: "user-1".into(),
                    phase_id: None,
                    title: "title".into(),
                    description: "desc".into(),
                    issue_type: mcb_domain::entities::project::IssueType::Task,
                    status: mcb_domain::entities::project::IssueStatus::Open,
                    priority: 1,
                    assignee: None,
                    labels: vec![],
                    estimated_minutes: None,
                    actual_minutes: None,
                    notes: String::new(),
                    design: String::new(),
                    parent_issue_id: None,
                    created_at: 0,
                    updated_at: 0,
                    closed_at: None,
                    closed_reason: String::new(),
                });
        }
        let handler = IssueEntityHandler::new(service);
        let args = IssueEntityArgs {
            action: IssueEntityAction::Get,
            resource: IssueEntityResource::Issue,
            id: Some("i1".into()),
            org_id: None,
            project_id: None,
            issue_id: None,
            label_id: None,
            data: None,
        };
        let result = handler.handle(Parameters(args)).await.expect("handle ok");
        assert!(!result.content.is_empty());
    }
}
