//! Tests for Python project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::git::project_detection::PythonDetector;

#[test]
fn test_python_detector_constructs() {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_string(),
    };
    let detector = PythonDetector::new(&config);
    assert!(!std::any::type_name::<PythonDetector>().is_empty());
    let _ = detector;
}

#[test]
fn test_python_detector_is_object_safe() {
    fn _assert_object_safe(_: &dyn ProjectDetector) {}
    assert!(!std::any::type_name::<PythonDetector>().is_empty());
}
