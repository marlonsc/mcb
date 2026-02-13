//! Unit tests for jobs domain ports.

use mcb_domain::ports::{Job, JobCounts, JobStatus, JobType};
use mcb_domain::value_objects::OperationId;

#[test]
fn job_status_terminal_check() {
    assert!(!JobStatus::Queued.is_terminal());
    assert!(!JobStatus::Running.is_terminal());
    assert!(JobStatus::Completed.is_terminal());
    assert!(JobStatus::Failed("oops".into()).is_terminal());
    assert!(JobStatus::Cancelled.is_terminal());
}

#[test]
fn job_status_active_check() {
    assert!(JobStatus::Queued.is_active());
    assert!(JobStatus::Running.is_active());
    assert!(!JobStatus::Completed.is_active());
    assert!(!JobStatus::Failed("error".into()).is_active());
    assert!(!JobStatus::Cancelled.is_active());
}

#[test]
fn job_type_display() {
    assert_eq!(JobType::Indexing.to_string(), "indexing");
    assert_eq!(JobType::Validation.to_string(), "validation");
    assert_eq!(JobType::Analysis.to_string(), "analysis");
    assert_eq!(
        JobType::Custom("my-job".into()).to_string(),
        "custom:my-job"
    );
}

#[test]
fn new_job_defaults() {
    let job = Job::new(
        OperationId::new("test-123"),
        JobType::Indexing,
        "Index codebase",
    );
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.progress_percent, 0);
    assert_eq!(job.processed_items, 0);
    assert_eq!(job.total_items, 0);
    assert!(job.started_at.is_none());
    assert!(job.completed_at.is_none());
    assert!(job.result.is_none());
}

#[test]
fn job_counts_default() {
    let counts = JobCounts::default();
    assert_eq!(counts.queued, 0);
    assert_eq!(counts.running, 0);
    assert_eq!(counts.completed, 0);
    assert_eq!(counts.failed, 0);
    assert_eq!(counts.cancelled, 0);
}
