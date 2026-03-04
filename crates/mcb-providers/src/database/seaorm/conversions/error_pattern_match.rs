//! Error pattern match entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::error_pattern_match;
use mcb_domain::entities::memory::ErrorPatternMatch;

crate::impl_conversion!(error_pattern_match, ErrorPatternMatch,
    direct: [id, pattern_id, observation_id, confidence, matched_at, resolved_at],
    opt_bools: [resolution_successful],
    opt_int_casts: [solution_applied]
);
