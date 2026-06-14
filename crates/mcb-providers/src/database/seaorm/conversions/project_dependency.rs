//! Project dependency entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project_dependency;
use mcb_domain::entities::project::{DependencyType, ProjectDependency};

crate::impl_conversion!(project_dependency, ProjectDependency,
    direct: [id, from_issue_id, to_issue_id, created_at],
    enums: { dependency_type: DependencyType = DependencyType::RelatesTo }
);
