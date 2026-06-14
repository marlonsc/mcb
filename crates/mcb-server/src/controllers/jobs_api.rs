//! Jobs API controller — returns indexing and validation operation status as JSON.

use std::collections::HashMap;

use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::ports::{
    IndexingOperation, IndexingOperationStatus, ValidationOperation, ValidationStatus,
};
use mcb_domain::value_objects::ids::OperationId;

/// Returns a summary of all indexing and validation operations.
///
/// Calls `IndexingOperationsInterface::get_operations()` (sync) and
/// `ValidationOperationsInterface::get_operations()` (sync) on the shared
/// trackers from `McbState`.
///
/// # Errors
///
/// Fails when operations cannot be serialized.
pub async fn jobs(Extension(state): Extension<McbState>) -> Result<Response> {
    let indexing_ops = state.indexing_ops.get_operations();
    let validation_ops = state.validation_ops.get_operations();

    let indexing_running = indexing_ops
        .values()
        .filter(|op| {
            matches!(
                op.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count();
    let validation_running = validation_ops
        .values()
        .filter(|op| {
            matches!(
                op.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
        .count();

    let total = indexing_ops.len() + validation_ops.len();
    let running = indexing_running + validation_running;
    let queued = count_queued(&indexing_ops, &validation_ops);

    // Combine all operations into a single jobs array for API consumers
    let mut jobs: Vec<serde_json::Value> = indexing_ops
        .values()
        .map(|op| serde_json::to_value(op).unwrap_or_default())
        .collect();
    jobs.extend(
        validation_ops
            .values()
            .map(|op| serde_json::to_value(op).unwrap_or_default()),
    );

    format::json(serde_json::json!({
        "total": total,
        "running": running,
        "queued": queued,
        "jobs": jobs,
        "indexing_operations": indexing_ops.values().collect::<Vec<_>>(),
        "validation_operations": validation_ops.values().collect::<Vec<_>>(),
    }))
}

/// Count operations in a queued/starting state across both trackers.
fn count_queued(
    indexing_ops: &HashMap<OperationId, IndexingOperation>,
    validation_ops: &HashMap<OperationId, ValidationOperation>,
) -> usize {
    validation_ops
        .values()
        .filter(|op| matches!(op.status, ValidationStatus::Queued))
        .count()
        + indexing_ops
            .values()
            .filter(|op| matches!(op.status, IndexingOperationStatus::Starting))
            .count()
}

/// Registers jobs API routes.
#[must_use]
pub fn routes() -> Routes {
    Routes::new().prefix("jobs").add("/", get(jobs))
}
