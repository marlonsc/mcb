//! Benchmarks for mcb-validate validation operations
//!
//! Run with: cargo bench -p mcb-validate

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mcb_validate::ValidationConfig;
use mcb_validate::ast::UnwrapDetector;
use mcb_validate::clean_architecture::CleanArchitectureValidator;
use mcb_validate::duplication::{DuplicationAnalyzer, DuplicationThresholds, tokenize_source};
use mcb_validate::generic_reporter::GenericReporter;
use mcb_validate::violation_trait::Violation;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use tempfile::TempDir;

/// Sample Rust code for benchmarking
const SAMPLE_RUST_CODE: &str = r#"
//! Sample module for benchmarking

use std::collections::HashMap;
use std::sync::Arc;
use rstest::rstest;

/// A user entity with identity
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(name: String, email: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            email,
            metadata: HashMap::new(),
        }
    }

    /// Get user's display name
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Set a metadata field
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// A service for user operations
pub struct UserService {
    users: Arc<HashMap<uuid::Uuid, User>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: Arc::new(HashMap::new()),
        }
    }

    pub fn find_user(&self, id: &uuid::Uuid) -> Option<&User> {
        None
    }

    pub fn create_user(&mut self, name: String, email: String) -> User {
        User::new(name, email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[test]
    fn test_user_creation() {
        let user = User::new("Alice".to_string(), "alice@example.com".to_string());
        assert_eq!(user.display_name(), "Alice");
    }
}
"#;

/// Code with duplication for fingerprinting benchmarks
const DUPLICATED_CODE: &str = r#"
fn calculate_sum(items: &[i32]) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    total
}

fn compute_total(values: &[i32]) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}

fn aggregate_items(data: &[i32]) -> i32 {
    let mut result = 0;
    for d in data {
        result += d;
    }
    result
}
"#;

/// Code with unwrap calls for detection benchmarks
const CODE_WITH_UNWRAPS: &str = r#"
pub fn risky_function() {
    let opt: Option<i32> = Some(42);
    let value = opt.unwrap();
    let result = Some("hello").expect("failed");
    let data = vec![1, 2, 3];
    let first = data.get(0).unwrap();
}
"#;

fn create_test_workspace() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    // Create workspace structure
    fs::create_dir_all(root.join("crates/mcb-domain/src/entities")).unwrap();
    fs::create_dir_all(root.join("crates/mcb-domain/src/value_objects")).unwrap();
    fs::create_dir_all(root.join("crates/mcb-providers/src")).unwrap();
    fs::create_dir_all(root.join("crates/mcb-server/src/handlers")).unwrap();

    // Create Cargo.toml
    let cargo_toml = r#"
[workspace]
members = ["crates/mcb-domain", "crates/mcb-providers", "crates/mcb-server"]
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();

    // Write sample code
    fs::write(
        root.join("crates/mcb-domain/src/entities/user.rs"),
        SAMPLE_RUST_CODE,
    )
    .unwrap();
    fs::write(
        root.join("crates/mcb-domain/src/entities/mod.rs"),
        "pub mod user;",
    )
    .unwrap();
    fs::write(
        root.join("crates/mcb-domain/src/lib.rs"),
        "pub mod entities;\npub mod value_objects;",
    )
    .unwrap();
    fs::write(
        root.join("crates/mcb-domain/src/value_objects/mod.rs"),
        "// value objects",
    )
    .unwrap();

    (dir, root)
}

/// Benchmark unwrap detection
fn bench_unwrap_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("unwrap_detection");

    group.throughput(Throughput::Bytes(CODE_WITH_UNWRAPS.len() as u64));

    group.bench_function("detect_unwraps", |b| {
        b.iter(|| {
            let mut detector = UnwrapDetector::new().unwrap();
            detector.detect_in_content(black_box(CODE_WITH_UNWRAPS), "test.rs")
        })
    });

    // Larger code
    let large_code = CODE_WITH_UNWRAPS.repeat(10);
    group.throughput(Throughput::Bytes(large_code.len() as u64));

    group.bench_function("detect_unwraps_10x", |b| {
        b.iter(|| {
            let mut detector = UnwrapDetector::new().unwrap();
            detector.detect_in_content(black_box(&large_code), "test.rs")
        })
    });

    group.finish();
}

/// Benchmark tokenization for duplication detection
fn bench_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");

    group.throughput(Throughput::Bytes(SAMPLE_RUST_CODE.len() as u64));

    group.bench_function("tokenize_rust", |b| {
        b.iter(|| tokenize_source(black_box(SAMPLE_RUST_CODE), "rust"))
    });

    // Benchmark with duplicated code
    group.throughput(Throughput::Bytes(DUPLICATED_CODE.len() as u64));

    group.bench_function("tokenize_duplicates", |b| {
        b.iter(|| tokenize_source(black_box(DUPLICATED_CODE), "rust"))
    });

    group.finish();
}

/// Benchmark duplication analysis
fn bench_duplication_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("duplication_analysis");

    let (_dir, root) = create_test_workspace();

    // Write duplicated code to a file
    let lib_path = root.join("crates/mcb-domain/src/lib.rs");
    fs::write(&lib_path, DUPLICATED_CODE).unwrap();

    let analyzer = DuplicationAnalyzer::with_thresholds(DuplicationThresholds::default());
    let files = vec![lib_path];

    group.bench_function("analyze_files", |b| {
        b.iter(|| analyzer.analyze_files(black_box(&files)))
    });

    group.finish();
}

/// Benchmark architecture validation
fn bench_architecture_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("architecture_validation");

    let (_dir, root) = create_test_workspace();

    let validator = CleanArchitectureValidator::new(&root);

    group.bench_function("validate_workspace", |b| {
        b.iter(|| validator.validate_all())
    });

    group.finish();
}

/// Benchmark report generation
fn bench_report_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("report_generation");

    let (_dir, root) = create_test_workspace();

    // Empty violations
    let empty_violations: Vec<Box<dyn Violation>> = vec![];

    group.bench_function("report_empty", |b| {
        b.iter(|| {
            GenericReporter::create_report(black_box(&empty_violations), black_box(root.clone()))
        })
    });

    group.finish();
}

/// Benchmark ValidationConfig creation
fn bench_config_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");

    let (_dir, root) = create_test_workspace();

    group.bench_function("create_config", |b| {
        b.iter(|| ValidationConfig::new(black_box(&root)))
    });

    group.bench_function("create_config_with_excludes", |b| {
        b.iter(|| {
            ValidationConfig::new(black_box(&root))
                .with_exclude_pattern("target/")
                .with_exclude_pattern("**/tests/**")
                .with_exclude_pattern("**/benches/**")
        })
    });

    group.finish();
}

/// Benchmark with different code sizes
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    for multiplier in [1, 5, 10, 20] {
        let code = SAMPLE_RUST_CODE.repeat(multiplier);

        group.throughput(Throughput::Bytes(code.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("tokenize", multiplier),
            &code,
            |b, code| b.iter(|| tokenize_source(black_box(code), "rust")),
        );

        group.bench_with_input(
            BenchmarkId::new("unwrap_detect", multiplier),
            &code,
            |b, code| {
                b.iter(|| {
                    let mut detector = UnwrapDetector::new().unwrap();
                    detector.detect_in_content(black_box(code), "test.rs")
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_unwrap_detection,
    bench_tokenization,
    bench_duplication_analysis,
    bench_architecture_validation,
    bench_report_generation,
    bench_config_creation,
    bench_scalability,
);

criterion_main!(benches);
