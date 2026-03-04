//! Repository entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::repository;
use mcb_domain::entities::Repository;
use mcb_domain::entities::repository::VcsType;

crate::impl_conversion!(repository, Repository,
    direct: [id, org_id, project_id, name, url, local_path, created_at, updated_at],
    enums: { vcs_type: VcsType = VcsType::Git },
    not_set: [origin_context]
);
