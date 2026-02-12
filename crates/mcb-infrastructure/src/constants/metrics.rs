/// Constant value for `METRICS_COLLECTION_INTERVAL_SECS`.
pub const METRICS_COLLECTION_INTERVAL_SECS: u64 = 60;
/// Constant value for `METRICS_PREFIX`.
pub const METRICS_PREFIX: &str = "mcb";
/// Constant value for `METRICS_LATENCY_BUCKETS`.
pub const METRICS_LATENCY_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];
/// Constant value for `METRICS_BATCH_SIZE_BUCKETS`.
pub const METRICS_BATCH_SIZE_BUCKETS: &[f64] =
    &[1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
