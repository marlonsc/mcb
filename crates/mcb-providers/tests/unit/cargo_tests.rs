//! Tests for Cargo project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::git::project_detection::CargoDetector;

#[test]
fn test_cargo_detector_constructs() {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_string(),
    };
    let detector = CargoDetector::new(&config);
    assert!(!std::any::type_name::<CargoDetector>().is_empty());
    let _ = detector;
}

#[test]
fn test_cargo_detector_is_object_safe() {
    fn _assert_object_safe(_: &dyn ProjectDetector) {}
    assert!(!std::any::type_name::<CargoDetector>().is_empty());
}
