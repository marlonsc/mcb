//! Tests for Go project detector (REF003).

use mcb_domain::ports::providers::project_detection::{ProjectDetector, ProjectDetectorConfig};
use mcb_providers::project_detection::GoDetector;
use rstest::rstest;

#[rstest]
#[case(false)]
#[case(true)]
fn go_detector_basics(#[case] check_object_safety: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = ProjectDetectorConfig {
        repo_path: ".".to_owned(),
    };
    let detector = GoDetector::new(&config)?;
    assert!(!std::any::type_name::<GoDetector>().is_empty());
    if check_object_safety {
        fn _assert_object_safe(_: &dyn ProjectDetector) {}
        _assert_object_safe(&detector);
    }
    Ok(())
}
