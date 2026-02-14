//! Unit tests for jobs domain ports.

use mcb_domain::ports::{Job, JobCounts, JobStatus, JobType};
use mcb_domain::value_objects::OperationId;
use rstest::rstest;

#[rstest]
#[case(JobStatus::Queued, false, true)]
#[case(JobStatus::Running, false, true)]
#[case(JobStatus::Completed, true, false)]
#[case(JobStatus::Failed("oops".into()), true, false)]
#[case(JobStatus::Cancelled, true, false)]
fn job_status_flags(
    #[case] status: JobStatus,
    #[case] expected_terminal: bool,
    #[case] expected_active: bool,
) {
    assert_eq!(status.is_terminal(), expected_terminal);
    assert_eq!(status.is_active(), expected_active);
}

#[rstest]
#[case(JobType::Indexing, "indexing")]
#[case(JobType::Validation, "validation")]
#[case(JobType::Analysis, "analysis")]
#[case(JobType::Custom("my-job".into()), "custom:my-job")]
fn job_type_display(#[case] job_type: JobType, #[case] expected: &str) {
    assert_eq!(job_type.to_string(), expected);
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
