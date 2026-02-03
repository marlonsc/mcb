use crate::entities::project::ProjectType;

/// Helper for retrieving the project name without needing methods on the domain type.
pub fn project_name(project: &ProjectType) -> &str {
    match project {
        ProjectType::Cargo { name, .. } => name,
        ProjectType::Npm { name, .. } => name,
        ProjectType::Python { name, .. } => name,
        ProjectType::Go { module, .. } => module,
        ProjectType::Maven { artifact_id, .. } => artifact_id,
    }
}

/// Helper to get the normalized type label for the project.
pub fn project_type_name(project: &ProjectType) -> &'static str {
    match project {
        ProjectType::Cargo { .. } => "cargo",
        ProjectType::Npm { .. } => "npm",
        ProjectType::Python { .. } => "python",
        ProjectType::Go { .. } => "go",
        ProjectType::Maven { .. } => "maven",
    }
}
