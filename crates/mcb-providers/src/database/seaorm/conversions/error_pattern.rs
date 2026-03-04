//! Error pattern entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::error_pattern;
use mcb_domain::entities::memory::{ErrorPattern, ErrorPatternCategory};

crate::impl_conversion!(error_pattern, ErrorPattern,
    direct: [id, project_id, pattern_signature, description, occurrence_count,
             first_seen_at, last_seen_at, embedding_id],
    enums: { category: ErrorPatternCategory = ErrorPatternCategory::Other },
    json_arrays: [solutions, affected_files, tags]
);
