//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::entities::project::ProjectType;

/// Helper for retrieving the project name without needing methods on the domain type.
#[must_use]
pub fn project_name(project: &ProjectType) -> &str {
    match project {
        ProjectType::Cargo { name, .. }
        | ProjectType::Npm { name, .. }
        | ProjectType::Python { name, .. } => name,
        ProjectType::Go { module, .. } => module,
        ProjectType::Maven { artifact_id, .. } => artifact_id,
    }
}
