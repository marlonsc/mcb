#![allow(missing_docs)]

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_yaml_files(dir: &Path, root: &Path, acc: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, root, acc)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }

        let rel = path.strip_prefix(root).map_err(|error| {
            std::io::Error::other(format!(
                "failed to strip prefix for {} from {}: {error}",
                path.display(),
                root.display()
            ))
        })?;
        acc.push(rel.to_path_buf());
    }
    Ok(())
}

fn escape_for_rust_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let rules_dir = manifest_dir.join("rules");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let output_file = out_dir.join("embedded_rules_gen.rs");

    println!("cargo:rerun-if-changed={}", rules_dir.display());

    let mut yaml_files = Vec::new();
    collect_yaml_files(&rules_dir, &rules_dir, &mut yaml_files)?;
    yaml_files.sort();

    let mut generated = String::from("static EMBEDDED_RULES: &[(&str, &str)] = &[\n");

    for rel in yaml_files {
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let full_path = rules_dir.join(&rel_str);
        let full_escaped = escape_for_rust_string(&full_path.to_string_lossy());
        let rel_escaped = escape_for_rust_string(&format!("rules/{rel_str}"));

        writeln!(
            generated,
            "    (\"{rel_escaped}\", include_str!(\"{full_escaped}\")),"
        )?;
    }

    generated.push_str("];\n");
    fs::write(&output_file, generated)?;
    Ok(())
}
