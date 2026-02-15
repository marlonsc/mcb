//! Admin jobs endpoints
//!
//! Provides endpoints for monitoring background job status.

use mcb_domain::ports::jobs::{Job, JobStatus, JobType};
use rocket::serde::json::Json;
use rocket::{State, get};
use serde::Serialize;

use crate::admin::handlers::AdminState;

/// Jobs status response (unified job tracking)
#[derive(Serialize)]
pub struct JobsStatusResponse {
    /// Total number of tracked jobs
    pub total: usize,
    /// Number of currently running jobs
    pub running: usize,
    /// Number of queued jobs
    pub queued: usize,
    /// Job details
    pub jobs: Vec<Job>,
}

/// List all background jobs
#[get("/jobs")]
pub fn get_jobs_status(state: &State<AdminState>) -> Json<JobsStatusResponse> {
    tracing::info!("get_jobs_status called");
    let operations = state.indexing.get_operations();

    let jobs = operations
        .values()
        .map(|op| {
            let progress = if op.total_files > 0 {
                ((op.processed_files as f64 / op.total_files as f64) * 100.0) as u8
            } else {
                0
            };
            Job {
                id: op.id,
                job_type: JobType::Indexing,
                label: op.collection.to_string(),
                status: JobStatus::Running,
                progress_percent: progress,
                processed_items: op.processed_files,
                total_items: op.total_files,
                current_item: op.current_file.clone(),
                created_at: op.started_at,
                started_at: Some(op.started_at),
                completed_at: None,
                result: None,
            }
        })
        .collect::<Vec<_>>();

    let running = jobs.len();
    Json(JobsStatusResponse {
        total: running,
        running,
        queued: 0,
        jobs,
    })
}
