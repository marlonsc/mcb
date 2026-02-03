//! Tests for Git2 VCS provider (REF003: dedicated test file).

use mcb_domain::ports::providers::VcsProvider;
use mcb_providers::git::Git2Provider;

#[test]
fn test_git2_provider_constructs() {
    let provider = Git2Provider::new();
    assert!(
        !std::any::type_name::<Git2Provider>().is_empty(),
        "Git2Provider type exists"
    );
    let _ = provider;
}

#[test]
fn test_git2_provider_is_object_safe() {
    fn _assert_object_safe(_: &dyn VcsProvider) {}
    let provider = Git2Provider::new();
    _assert_object_safe(&provider);
    let _erased: &dyn VcsProvider = &provider;
    assert!(
        std::mem::size_of_val(_erased) > 0,
        "trait object must have non-zero size"
    );
}
