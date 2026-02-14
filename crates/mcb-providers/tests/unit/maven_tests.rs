//! Tests for Maven project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::git::project_detection::MavenDetector;
use rstest::rstest;

#[rstest]
#[case(false)]
#[case(true)]
fn maven_detector_basics(#[case] check_object_safety: bool) {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_string(),
    };
    let detector = MavenDetector::new(&config);
    assert!(!std::any::type_name::<MavenDetector>().is_empty());
    if check_object_safety {
        fn _assert_object_safe(_: &dyn ProjectDetector) {}
        _assert_object_safe(&detector);
    }
}
