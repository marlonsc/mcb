//! HTTP metrics collection and Prometheus exposition.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use mcb_domain::ports::ReadinessReport;

const LATENCY_BUCKETS: [f64; 8] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.5, 1.0, 5.0];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RequestKey {
    method: String,
    path: String,
    status: u16,
}

#[derive(Debug, Default)]
struct RequestMetric {
    count: u64,
    duration_sum_seconds: f64,
    buckets: [u64; LATENCY_BUCKETS.len()],
}

/// Process-local HTTP metrics registry.
#[derive(Debug, Clone, Default)]
pub struct ServerMetrics {
    requests: Arc<Mutex<BTreeMap<RequestKey, RequestMetric>>>,
}

impl ServerMetrics {
    /// Record one completed HTTP request.
    pub fn record(&self, method: &str, path: &str, status: u16, duration_seconds: f64) {
        let Ok(mut requests) = self.requests.lock() else {
            return;
        };
        let metric = requests.entry(RequestKey {
            method: method.to_owned(),
            path: path.to_owned(),
            status,
        });
        let metric = metric.or_default();
        metric.count = metric.count.saturating_add(1);
        metric.duration_sum_seconds += duration_seconds;
        for (index, upper_bound) in LATENCY_BUCKETS.iter().enumerate() {
            if duration_seconds <= *upper_bound {
                metric.buckets[index] = metric.buckets[index].saturating_add(1);
            }
        }
    }

    /// Render the registry and live runtime gauges in Prometheus text format.
    #[must_use]
    pub fn render(
        &self,
        readiness: &ReadinessReport,
        active_indexing_jobs: usize,
        active_validation_jobs: usize,
    ) -> String {
        let mut output = String::from(
            "# HELP mcb_http_requests_total Completed HTTP requests.\n\
# TYPE mcb_http_requests_total counter\n\
# HELP mcb_http_request_duration_seconds HTTP request duration in seconds.\n\
# TYPE mcb_http_request_duration_seconds histogram\n",
        );
        if let Ok(requests) = self.requests.lock() {
            for (key, metric) in requests.iter() {
                write_request_metric(&mut output, key, metric);
            }
        }
        let readiness_value = u8::from(readiness.ready);
        let _ = write!(
            output,
            "# HELP mcb_ready Whether all required runtime dependencies are usable.\n\
# TYPE mcb_ready gauge\n\
mcb_ready {readiness_value}\n\
# HELP mcb_dependency_ready Whether a required runtime dependency is usable.\n\
# TYPE mcb_dependency_ready gauge\n"
        );
        for dependency in &readiness.dependencies {
            let value = u8::from(dependency.ready);
            let _ = writeln!(
                output,
                "mcb_dependency_ready{{dependency=\"{}\"}} {value}",
                dependency.name
            );
        }
        let _ = write!(
            output,
            "# HELP mcb_active_indexing_jobs Current active indexing jobs.\n\
# TYPE mcb_active_indexing_jobs gauge\n\
mcb_active_indexing_jobs {active_indexing_jobs}\n\
# HELP mcb_active_validation_jobs Current active validation jobs.\n\
# TYPE mcb_active_validation_jobs gauge\n\
mcb_active_validation_jobs {active_validation_jobs}\n"
        );
        output
    }
}

fn write_request_metric(output: &mut String, key: &RequestKey, metric: &RequestMetric) {
    let labels = format!(
        "method=\"{}\",path=\"{}\",status=\"{}\"",
        key.method, key.path, key.status
    );
    let _ = writeln!(
        output,
        "mcb_http_requests_total{{{labels}}} {}",
        metric.count
    );
    for (index, upper_bound) in LATENCY_BUCKETS.iter().enumerate() {
        let _ = writeln!(
            output,
            "mcb_http_request_duration_seconds_bucket{{{labels},le=\"{upper_bound}\"}} {}",
            metric.buckets[index]
        );
    }
    let _ = writeln!(
        output,
        "mcb_http_request_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {}",
        metric.count
    );
    let _ = writeln!(
        output,
        "mcb_http_request_duration_seconds_sum{{{labels}}} {}",
        metric.duration_sum_seconds
    );
    let _ = writeln!(
        output,
        "mcb_http_request_duration_seconds_count{{{labels}}} {}",
        metric.count
    );
}

/// Record HTTP request count, status, and latency.
pub async fn track_http(metrics: ServerMetrics, request: Request, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(request).await;
    metrics.record(
        &method,
        &path,
        response.status().as_u16(),
        started.elapsed().as_secs_f64(),
    );
    response
}
