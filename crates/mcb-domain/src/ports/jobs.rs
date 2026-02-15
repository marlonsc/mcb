//! Jobs Service Domain Ports
//!
//! Defines generic job management interfaces that abstract over specific
//! operation types (indexing, validation, analysis, etc.).
//!
//! This is the unified task tracking system. Individual operation types
//! (like `IndexingOperation`) map to `Job` instances with a specific `JobType`.

use std::collections::HashMap;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::value_objects::OperationId;

// ============================================================================
// Job Types & Status
// ============================================================================

/// Unique identifier for a job (wraps OperationId for domain consistency)
pub type JobId = OperationId;

/// The type of work a job performs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display)]
pub enum JobType {
    /// Codebase indexing operation
    #[display("indexing")]
    Indexing,
    /// Architectural validation operation
    #[display("validation")]
    Validation,
    /// Code analysis / complexity assessment
    #[display("analysis")]
    Analysis,
    /// Custom job type with a user-defined label
    #[display("custom:{_0}")]
    Custom(String),
}

/// Lifecycle status of a job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    /// Job is waiting to be picked up
    Queued,
    /// Job is currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job terminated with an error
    Failed(String),
    /// Job was manually cancelled
    Cancelled,
}

impl JobStatus {
    /// Returns `true` if the job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed(_) | Self::Cancelled)
    }

    /// Returns `true` if the job is actively running
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

/// Result metadata attached to a completed job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Summary message describing the outcome
    pub summary: String,
    /// Number of items successfully processed
    pub items_processed: usize,
    /// Number of items that failed processing
    pub items_failed: usize,
    /// Arbitrary key/value metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Job Entity
// ============================================================================

/// A generic background job tracked by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for the job
    pub id: JobId,
    /// What kind of work this job performs
    pub job_type: JobType,
    /// Human-readable label for the job
    pub label: String,
    /// Current lifecycle status
    pub status: JobStatus,
    /// Progress as a percentage (0..=100)
    pub progress_percent: u8,
    /// Number of items processed so far
    pub processed_items: usize,
    /// Total number of items to process (0 = unknown)
    pub total_items: usize,
    /// Description of the item currently being processed
    pub current_item: Option<String>,
    /// When the job was created/queued (Unix epoch seconds)
    pub created_at: i64,
    /// When the job started running (Unix epoch seconds, if applicable)
    pub started_at: Option<i64>,
    /// When the job reached a terminal state (Unix epoch seconds, if applicable)
    pub completed_at: Option<i64>,
    /// Result metadata (populated on completion)
    pub result: Option<JobResult>,
}

impl Job {
    /// Create a new job in `Queued` status
    pub fn new(id: JobId, job_type: JobType, label: impl Into<String>) -> Self {
        Self {
            id,
            job_type,
            label: label.into(),
            status: JobStatus::Queued,
            progress_percent: 0,
            processed_items: 0,
            total_items: 0,
            current_item: None,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            result: None,
        }
    }
}

// ============================================================================
// Job Manager Interface
// ============================================================================

/// Progress update payload for advancing a running job
#[derive(Debug, Clone)]
pub struct JobProgressUpdate {
    /// Description of the current item being processed
    pub current_item: Option<String>,
    /// Number of items processed so far
    pub processed_items: usize,
    /// Total number of items to process
    pub total_items: usize,
}

/// Interface for managing the lifecycle of background jobs.
///
/// Implementations track creation, progress, completion, and cancellation
/// of jobs across all job types.
pub trait JobManagerInterface: Send + Sync {
    /// List all tracked jobs, optionally filtered by type
    fn list_jobs(&self, job_type: Option<&JobType>) -> Vec<Job>;

    /// Get a specific job by ID
    fn get_job(&self, job_id: &JobId) -> Option<Job>;

    /// Submit a new job and return its assigned ID
    fn submit_job(&self, job_type: JobType, label: &str, total_items: usize) -> JobId;

    /// Mark a queued job as running
    fn start_job(&self, job_id: &JobId);

    /// Update progress on a running job
    fn update_progress(&self, job_id: &JobId, update: JobProgressUpdate);

    /// Mark a job as successfully completed
    fn complete_job(&self, job_id: &JobId, result: Option<JobResult>);

    /// Mark a job as failed with an error message
    fn fail_job(&self, job_id: &JobId, error: &str);

    /// Cancel a queued or running job
    fn cancel_job(&self, job_id: &JobId);

    /// Get counts of jobs by status (for dashboard summaries)
    fn job_counts(&self) -> JobCounts;
}

/// Summary counts of jobs grouped by status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobCounts {
    /// Number of jobs waiting to start
    pub queued: usize,
    /// Number of actively running jobs
    pub running: usize,
    /// Number of successfully completed jobs
    pub completed: usize,
    /// Number of failed jobs
    pub failed: usize,
    /// Number of cancelled jobs
    pub cancelled: usize,
}
