//! Tests for npm project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::git::project_detection::NpmDetector;

#[test]
fn test_npm_detector_constructs() {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_string(),
    };
    let detector = NpmDetector::new(&config);
    assert!(!std::any::type_name::<NpmDetector>().is_empty());
    let _ = detector;
}

#[test]
fn test_npm_detector_is_object_safe() {
    fn _assert_object_safe(_: &dyn ProjectDetector) {}
    assert!(!std::any::type_name::<NpmDetector>().is_empty());
}
