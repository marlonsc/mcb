//! Unit tests for Git VCS provider — repo open/close/id tests.

use rstest::rstest;
use std::path::Path;

use mcb_domain::test_utils::TestResult;

use super::common::{create_test_repo, vcs_provider};

#[rstest]
#[case(false)]
#[case(true)]
fn git_provider_basics(#[case] check_object_safety: bool) {
    let provider = vcs_provider().expect("vcs provider should resolve");
    if check_object_safety {
        fn _assert_object_safe(_: &dyn mcb_domain::ports::VcsProvider) {}
        _assert_object_safe(provider.as_ref());
        let _erased: &dyn mcb_domain::ports::VcsProvider = provider.as_ref();
        assert_eq!(
            std::mem::size_of::<&dyn mcb_domain::ports::VcsProvider>(),
            2 * std::mem::size_of::<usize>(),
            "trait object reference should be a fat pointer"
        );
    }
}

#[rstest]
#[tokio::test]
async fn open_repository() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = vcs_provider()?;

    let repo = provider.open_repository(dir.path()).await?;
    assert!(!repo.id().as_str().is_empty());
    assert!(!repo.branches().is_empty());
    Ok(())
}

#[rstest]
#[case("/nonexistent/path")]
#[case("/this/definitely/does/not/exist")]
#[tokio::test]
async fn open_repository_not_found(#[case] repo_path: &str) {
    let provider = vcs_provider().expect("vcs provider should resolve");
    let result = provider.open_repository(Path::new(repo_path)).await;
    let err = result.expect_err("should fail for non-existent path");
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("not found"),
        "expected 'not found' in error message, got: {err}"
    );
}

#[rstest]
#[tokio::test]
async fn repository_id_stable() -> TestResult<()> {
    let dir = create_test_repo().await?;
    let provider = vcs_provider()?;

    let repo1 = provider.open_repository(dir.path()).await?;
    let repo2 = provider.open_repository(dir.path()).await?;

    assert_eq!(
        provider.repository_id(&repo1),
        provider.repository_id(&repo2)
    );
    Ok(())
}
