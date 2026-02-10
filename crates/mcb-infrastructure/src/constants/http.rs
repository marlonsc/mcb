/// Default HTTP server port.
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default HTTPS server port.
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Default server host address (localhost).
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// Request timeout in seconds.
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Connection timeout in seconds.
pub const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Maximum HTTP request body size in bytes (10 MB).
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;

/// HTTP endpoint path for health checks.
pub const HEALTH_CHECK_PATH: &str = "/health";

/// HTTP endpoint path for metrics.
pub const METRICS_PATH: &str = "/metrics";

/// HTTP request timeout in seconds.
pub const HTTP_REQUEST_TIMEOUT_SECS: u64 = 30;

/// HTTP client idle timeout in seconds.
pub const HTTP_CLIENT_IDLE_TIMEOUT_SECS: u64 = 90;

/// HTTP keep-alive timeout in seconds.
pub const HTTP_KEEPALIVE_SECS: u64 = 60;

/// Maximum number of idle connections per host.
pub const HTTP_MAX_IDLE_PER_HOST: usize = 10;

/// HTTP header name for Content-Type.
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";

/// HTTP header name for Accept.
pub const HTTP_HEADER_ACCEPT: &str = "Accept";

/// HTTP header name for User-Agent.
pub const HTTP_HEADER_USER_AGENT: &str = "User-Agent";

/// MIME type for JSON content.
pub const CONTENT_TYPE_JSON: &str = "application/json";

/// Default CORS origin (allow all).
pub const DEFAULT_CORS_ORIGIN: &str = "*";
