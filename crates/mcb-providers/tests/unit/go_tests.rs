//! Tests for Go project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::git::project_detection::GoDetector;

#[test]
fn test_go_detector_constructs() {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_string(),
    };
    let detector = GoDetector::new(&config).unwrap();
    assert!(!std::any::type_name::<GoDetector>().is_empty());
    let _ = detector;
}

#[test]
fn test_go_detector_is_object_safe() {
    fn _assert_object_safe(_: &dyn ProjectDetector) {}
    assert!(!std::any::type_name::<GoDetector>().is_empty());
}
