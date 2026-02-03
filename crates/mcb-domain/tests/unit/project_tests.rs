//! Tests for project entity (REF003: dedicated test file).

use mcb_domain::entities::project::{DetectedProject, ProjectType};

#[test]
fn test_project_type_cargo() {
    let pt = ProjectType::Cargo {
        name: "foo".to_string(),
        version: "0.1.0".to_string(),
        dependencies: vec![],
    };
    match &pt {
        ProjectType::Cargo { name, version, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(version, "0.1.0");
        }
        _ => panic!("expected Cargo"),
    }
}

#[test]
fn test_detected_project_has_path_and_id() {
    let p = DetectedProject {
        id: "proj-1".to_string(),
        path: "crates/foo".to_string(),
        project_type: ProjectType::Cargo {
            name: "foo".to_string(),
            version: "0.1.0".to_string(),
            dependencies: vec![],
        },
        parent_repo_id: None,
    };
    assert_eq!(p.id, "proj-1");
    assert_eq!(p.path, "crates/foo");
}
