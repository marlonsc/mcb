//! Tests for Python project detector (REF003).

use mcb_domain::ports::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::project_detection::PythonDetector;
use rstest::rstest;

#[rstest]
#[case(false)]
#[case(true)]
fn python_detector_basics(#[case] check_object_safety: bool) {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_owned(),
    };
    let detector = PythonDetector::new(&config);
    assert!(!std::any::type_name::<PythonDetector>().is_empty());
    if check_object_safety {
        fn _assert_object_safe(_: &dyn ProjectDetector) {}
        _assert_object_safe(&detector);
    }
}
