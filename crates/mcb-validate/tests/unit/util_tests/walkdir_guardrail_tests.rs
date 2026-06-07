use rstest::rstest;
use std::path::PathBuf;

#[rstest]
#[test]
fn walkdir_new_is_limited_to_run_context_inventory() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let mut stack = vec![src_root.clone()];
    let mut matches = Vec::new();

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).expect("read_dir should succeed");
        for entry in entries {
            let entry = entry.expect("directory entry should be readable");
            let path = entry.path();
            let file_type = entry.file_type().expect("file_type should be readable");

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if !file_type.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            let content = std::fs::read_to_string(&path).expect("source file should be readable");
            if content.contains("WalkDir::new(") {
                let relative = path
                    .strip_prefix(&crate_root)
                    .expect("path should be under crate root")
                    .to_string_lossy()
                    .replace('\\', "/");
                matches.push(relative);
            }
        }
    }

    matches.sort();
    let allowed = ["src/run_context.rs"];
    assert_eq!(matches, allowed);
}
