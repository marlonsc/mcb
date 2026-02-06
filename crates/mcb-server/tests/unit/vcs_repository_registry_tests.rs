use mcb_server::vcs_repository_registry::{lookup_repository_path, record_repository};
use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::tempdir;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn with_temp_config_dir<T>(f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let temp = tempdir().expect("tempdir");
    let prev = env::var("XDG_CONFIG_HOME").ok();
    // SAFETY: We hold the ENV_LOCK mutex, ensuring exclusive access to environment variables
    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp.path());
    }
    let result = f();
    if let Some(value) = prev {
        // SAFETY: We hold the ENV_LOCK mutex, ensuring exclusive access to environment variables
        unsafe {
            env::set_var("XDG_CONFIG_HOME", value);
        }
    } else {
        // SAFETY: We hold the ENV_LOCK mutex, ensuring exclusive access to environment variables
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    result
}

#[test]
fn test_record_and_lookup_repository() {
    with_temp_config_dir(|| {
        let repo_id = "repo-123";
        let repo_path = PathBuf::from("/tmp/repo");
        record_repository(repo_id, &repo_path).expect("record repository");
        let loaded = lookup_repository_path(repo_id).expect("lookup repository");
        assert_eq!(loaded, repo_path);
    });
}

#[test]
fn test_lookup_missing_repository_returns_error() {
    with_temp_config_dir(|| {
        let err = lookup_repository_path("missing-repo").unwrap_err();
        assert!(
            err.to_string().contains("Repository id not found"),
            "unexpected error: {err}"
        );
    });
}
