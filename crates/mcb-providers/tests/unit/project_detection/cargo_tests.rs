//! Tests for Cargo project detector (REF003).

use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::project_detection::CargoDetector;
use rstest::rstest;

#[rstest]
#[case(false)]
#[case(true)]
fn cargo_detector_basics(#[case] check_object_safety: bool) {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_owned(),
    };
    let detector = CargoDetector::new(&config);
    assert!(!std::any::type_name::<CargoDetector>().is_empty());
    if check_object_safety {
        fn _assert_object_safe(_: &dyn ProjectDetector) {}
        _assert_object_safe(&detector);
    }
}
