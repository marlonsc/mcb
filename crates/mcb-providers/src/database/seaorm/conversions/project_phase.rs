//! Project phase entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project_phase;
use mcb_domain::entities::project::{PhaseStatus, ProjectPhase};

crate::impl_conversion!(project_phase, ProjectPhase,
    direct: [id, project_id, name, description, started_at, completed_at, created_at, updated_at],
    enums: { status: PhaseStatus = PhaseStatus::Planned },
    int_casts: [sequence]
);
