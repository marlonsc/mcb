//! Tests for npm project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::project_detection::NpmDetector;
use rstest::rstest;

#[rstest]
#[case(false)]
#[case(true)]
fn npm_detector_basics(#[case] check_object_safety: bool) {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_owned(),
    };
    let detector = NpmDetector::new(&config);
    assert!(!std::any::type_name::<NpmDetector>().is_empty());
    if check_object_safety {
        fn _assert_object_safe(_: &dyn ProjectDetector) {}
        _assert_object_safe(&detector);
    }
}
