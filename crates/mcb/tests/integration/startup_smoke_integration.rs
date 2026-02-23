//! Startup smoke tests to catch runtime initialization failures in CI.
//!
//! Validates that the MCB binary handles database initialization correctly:
//! - Corrupted/incompatible databases are backed up and recreated
//! - DDL errors surface with actionable source context

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Locate the mcb binary from env or target directory.
///
/// # Panics
///
/// Panics if the binary cannot be found in any expected location.
fn get_mcb_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mcb") {
        return PathBuf::from(path);
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bin = format!("mcb{}", std::env::consts::EXE_SUFFIX);
    let debug_path = PathBuf::from(manifest_dir).join(format!("../../target/debug/{bin}"));
    if debug_path.exists() {
        return debug_path;
    }

    let release_path = PathBuf::from(manifest_dir).join(format!("../../target/release/{bin}"));
    if release_path.exists() {
        release_path
    } else {
        unreachable!(
            "mcb binary not found. Checked CARGO_BIN_EXE_mcb and target/debug|release/{bin} from {manifest_dir}"
        )
    }
}

fn unique_temp_path(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("mcb-startup-smoke-{name}-{stamp}"))
}

/// Spawn the MCB server process for testing.
///
/// # Panics
///
/// Panics if the process cannot be spawned.
fn spawn_mcb_serve(db_path: &std::path::Path) -> Child {
    Command::new(get_mcb_path())
        .arg("serve")
        .arg("--server")
        // Use Loco test environment (config/test.yaml with Tera template for DATABASE_URL)
        .env("LOCO_ENV", "test")
        .env(
            "DATABASE_URL",
            format!("sqlite://{}?mode=rwc", db_path.display()),
        )
        .env("RUST_LOG", "info")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| unreachable!("spawn mcb process: {e}"))
}

fn cleanup_temp_files(db_path: &std::path::Path, prefix: &str) {
    let _ = fs::remove_file(db_path);
    let Some(parent) = db_path.parent() else {
        return;
    };
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().contains(prefix) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[test]
#[ignore = "SeaORM migration path needs recovery logic for corrupt DBs"]
fn corrupted_db_is_backed_up_and_recreated() {
    let db_path = unique_temp_path("corrupt.db");
    fs::write(&db_path, b"this-is-not-a-valid-sqlite-database")
        .unwrap_or_else(|e| unreachable!("write corrupt db fixture: {e}"));

    let mut child = spawn_mcb_serve(&db_path);

    // Give it time to try init, fail, recover, and start
    // We can monitor stderr for the recovery message to exit early
    let stderr = child
        .stderr
        .take()
        .unwrap_or_else(|| unreachable!("capture stderr"));
    let reader = BufReader::new(stderr);
    let recovered = Arc::new(AtomicBool::new(false));
    let recovered_clone = Arc::clone(&recovered);

    let log_thread = thread::spawn(move || {
        for line in reader.lines().map_while(Result::ok) {
            if line.contains("backing up and recreating")
                || line.contains("Memory database recreated")
            {
                recovered_clone.store(true, Ordering::SeqCst);
                break;
            }
        }
    });

    // Wait up to 10 seconds for recovery
    for _ in 0..20 {
        if recovered.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }

    let _ = child.kill();
    let _ = child.wait();
    let _ = log_thread.join(); // This might hang if reader doesn't close, but child kill closes pipe

    let has_backup = db_path
        .parent()
        .and_then(|p| fs::read_dir(p).ok())
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("mcb-startup-smoke-corrupt.db")
                && e.file_name().to_string_lossy().contains(".bak.")
        });

    cleanup_temp_files(&db_path, "corrupt.db");

    assert!(
        has_backup || recovered.load(Ordering::SeqCst),
        "corrupt DB should trigger backup-and-recreate (SeaORM migration path may need recovery logic)"
    );
}

#[test]
#[ignore = "SeaORM migration path needs recovery logic for corrupt DBs"]
fn ddl_error_messages_include_source_context() {
    let db_path = unique_temp_path("ddl-ctx.db");
    fs::write(&db_path, vec![0u8; 100]).unwrap_or_else(|e| unreachable!("write invalid db: {e}"));

    let mut child = spawn_mcb_serve(&db_path);

    let stderr = child
        .stderr
        .take()
        .unwrap_or_else(|| unreachable!("capture stderr"));
    let reader = BufReader::new(stderr);

    let logs = Arc::new(std::sync::Mutex::new(String::new()));
    let found = Arc::new(AtomicBool::new(false));
    let logs_w = Arc::clone(&logs);
    let found_w = Arc::clone(&found);

    let log_thread = thread::spawn(move || {
        for line in reader.lines().map_while(Result::ok) {
            let has_target = line.contains("Memory database recreated")
                || line.contains("Observation storage error");
            {
                let mut buf = logs_w.lock().unwrap();
                buf.push_str(&line);
                buf.push('\n');
            }
            if has_target {
                found_w.store(true, Ordering::SeqCst);
                break;
            }
        }
    });

    for _ in 0..10 {
        if found.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }

    let _ = child.kill();
    let _ = child.wait();
    let _ = log_thread.join();

    let logs = logs.lock().unwrap();
    let recovery_worked = logs.contains("recreated") || logs.contains("backing up");
    let error_has_context = logs.contains("Observation storage error")
        || logs.contains("connect SQLite")
        || logs.contains("Failed to create database executor");

    cleanup_temp_files(&db_path, "ddl-ctx.db");

    assert!(
        recovery_worked || error_has_context,
        "should either recover or show actionable error; logs={logs}"
    );
}
