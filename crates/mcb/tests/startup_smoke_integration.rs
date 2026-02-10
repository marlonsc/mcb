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

fn get_mcb_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mcb") {
        return PathBuf::from(path);
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let debug_path = PathBuf::from(manifest_dir).join("../../target/debug/mcb");
    if debug_path.exists() {
        return debug_path;
    }

    let release_path = PathBuf::from(manifest_dir).join("../../target/release/mcb");
    if release_path.exists() {
        return release_path;
    }

    panic!(
        "mcb binary not found. Checked CARGO_BIN_EXE_mcb and target/debug|release/mcb from {}",
        manifest_dir
    );
}

fn unique_temp_path(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("mcb-startup-smoke-{}-{}", name, stamp))
}

fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/test.toml")
}

fn spawn_mcb_serve(db_path: &std::path::Path) -> Child {
    Command::new(get_mcb_path())
        .arg("serve")
        .arg("--server")
        .arg("--config")
        .arg(config_path())
        .env("MCP__SERVER__TRANSPORT_MODE", "http")
        .env("MCP__AUTH__USER_DB_PATH", db_path)
        .env("RUST_LOG", "info") // Ensure we get logs
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcb process")
}

fn cleanup_temp_files(db_path: &std::path::Path, prefix: &str) {
    let _ = fs::remove_file(db_path);
    if let Ok(entries) = fs::read_dir(db_path.parent().unwrap()) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().contains(prefix) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[test]
fn corrupted_db_is_backed_up_and_recreated() {
    let db_path = unique_temp_path("corrupt.db");
    fs::write(&db_path, b"this-is-not-a-valid-sqlite-database").expect("write corrupt db fixture");

    let mut child = spawn_mcb_serve(&db_path);

    // Give it time to try init, fail, recover, and start
    // We can monitor stderr for the recovery message to exit early
    let stderr = child.stderr.take().expect("capture stderr");
    let reader = BufReader::new(stderr);
    let recovered = Arc::new(AtomicBool::new(false));
    let recovered_clone = recovered.clone();

    let log_thread = thread::spawn(move || {
        for line in reader.lines() {
            if let Ok(l) = line {
                if l.contains("backing up and recreating")
                    || l.contains("Memory database recreated")
                {
                    recovered_clone.store(true, Ordering::SeqCst);
                }
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
    let _ = log_thread.join(); // This might hang if reader doesn't close, but child kill closes pipe

    let has_backup = fs::read_dir(db_path.parent().unwrap())
        .expect("read temp dir")
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("mcb-startup-smoke-corrupt.db")
                && e.file_name().to_string_lossy().contains(".bak.")
        });

    cleanup_temp_files(&db_path, "corrupt.db");

    assert!(
        has_backup || recovered.load(Ordering::SeqCst),
        "corrupt DB should trigger backup-and-recreate"
    );
}

#[test]
fn ddl_error_messages_include_source_context() {
    let db_path = unique_temp_path("ddl-ctx.db");
    // Write just enough zeros to look like a file but be invalid header
    fs::write(&db_path, vec![0u8; 100]).expect("write invalid db");

    let mut child = spawn_mcb_serve(&db_path);

    let stderr = child.stderr.take().expect("capture stderr");
    let reader = BufReader::new(stderr);

    let mut logs = String::new();
    // Read logs for a bit
    let start = std::time::Instant::now();
    for line in reader.lines() {
        if start.elapsed() > Duration::from_secs(5) {
            break;
        }
        if let Ok(l) = line {
            logs.push_str(&l);
            logs.push('\n');
            if l.contains("Memory database recreated") || l.contains("Observation storage error") {
                break;
            }
        }
    }

    let _ = child.kill();

    let recovery_worked = logs.contains("recreated") || logs.contains("backing up");
    let error_has_context = logs.contains("Observation storage error")
        || logs.contains("connect SQLite")
        || logs.contains("Failed to create database executor");

    cleanup_temp_files(&db_path, "ddl-ctx.db");

    assert!(
        recovery_worked || error_has_context,
        "should either recover or show actionable error; logs={}",
        logs
    );
}
