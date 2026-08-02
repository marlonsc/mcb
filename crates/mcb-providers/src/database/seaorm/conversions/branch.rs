//! Branch entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::branch;
use mcb_domain::entities::Branch;

crate::impl_conversion!(branch, Branch,
    direct: [id, org_id, repository_id, name, head_commit, upstream, created_at],
    bools: [is_default],
    not_set: [project_id, origin_context]
);
