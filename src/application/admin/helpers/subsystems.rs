//! Subsystem information helpers
//!
//! Builds SubsystemInfo structures from runtime data.

use crate::application::admin::types::{
    HealthCheck, ProviderInfo, SubsystemInfo, SubsystemMetrics, SubsystemStatus, SubsystemType,
};
use crate::domain::ports::admin::PerformanceMetricsData;
use crate::infrastructure::cache::CacheBackendConfig;
use crate::infrastructure::config::Config;
use crate::infrastructure::metrics::ProcessMetrics;

/// Build subsystem info from providers and metrics
pub fn build_subsystem_list(
    providers: &[ProviderInfo],
    process_metrics: &ProcessMetrics,
    perf: &PerformanceMetricsData,
    config: &Config,
    active_indexing_count: usize,
) -> Vec<SubsystemInfo> {
    let mut subsystems = Vec::new();

    // Calculate weights for proportional CPU/memory distribution
    let weights = calculate_weights(providers.len(), perf, config, active_indexing_count > 0);
    let total_memory_mb = process_metrics.memory / (1024 * 1024);

    // Add provider subsystems
    add_provider_subsystems(
        &mut subsystems,
        providers,
        process_metrics,
        perf,
        weights.per_provider,
        total_memory_mb,
    );

    // Add core subsystems
    add_core_subsystems(
        &mut subsystems,
        process_metrics,
        perf,
        config,
        &weights,
        total_memory_mb,
        active_indexing_count,
    );

    subsystems
}

/// Weights for distributing CPU/memory among subsystems
struct SubsystemWeights {
    search: f64,
    indexing: f64,
    cache: f64,
    http: f64,
    per_provider: f64,
}

fn calculate_weights(
    provider_count: usize,
    perf: &PerformanceMetricsData,
    config: &Config,
    is_indexing: bool,
) -> SubsystemWeights {
    // Base allocation percentages (adjust based on activity)
    let search_weight = if perf.total_queries > 0 { 0.30 } else { 0.10 };
    let indexing_weight = if is_indexing { 0.35 } else { 0.05 };
    let cache_weight = if config.cache.enabled { 0.15 } else { 0.0 };
    let http_weight = 0.10;

    let provider_weight: f64 = f64::max(
        1.0 - search_weight - indexing_weight - cache_weight - http_weight,
        0.0,
    );

    let per_provider_weight = if provider_count > 0 {
        provider_weight / provider_count as f64
    } else {
        0.0
    };

    SubsystemWeights {
        search: search_weight,
        indexing: indexing_weight,
        cache: cache_weight,
        http: http_weight,
        per_provider: per_provider_weight,
    }
}

fn add_provider_subsystems(
    subsystems: &mut Vec<SubsystemInfo>,
    providers: &[ProviderInfo],
    process_metrics: &ProcessMetrics,
    perf: &PerformanceMetricsData,
    per_provider_weight: f64,
    total_memory_mb: u64,
) {
    let embedding_count = providers
        .iter()
        .filter(|p| p.provider_type == "embedding")
        .count()
        .max(1);
    let vector_store_count = providers
        .iter()
        .filter(|p| p.provider_type == "vector_store")
        .count()
        .max(1);

    // Add embedding providers
    for provider in providers.iter().filter(|p| p.provider_type == "embedding") {
        subsystems.push(build_provider_subsystem(&ProviderSubsystemConfig {
            provider,
            subsystem_type: SubsystemType::Embedding,
            type_name: "Embedding Provider",
            process_metrics,
            perf,
            per_provider_weight,
            total_memory_mb,
            provider_count: embedding_count,
        }));
    }

    // Add vector store providers
    for provider in providers
        .iter()
        .filter(|p| p.provider_type == "vector_store")
    {
        subsystems.push(build_provider_subsystem(&ProviderSubsystemConfig {
            provider,
            subsystem_type: SubsystemType::VectorStore,
            type_name: "Vector Store",
            process_metrics,
            perf,
            per_provider_weight,
            total_memory_mb,
            provider_count: vector_store_count,
        }));
    }
}

/// Configuration for building a provider subsystem
struct ProviderSubsystemConfig<'a> {
    provider: &'a ProviderInfo,
    subsystem_type: SubsystemType,
    type_name: &'a str,
    process_metrics: &'a ProcessMetrics,
    perf: &'a PerformanceMetricsData,
    per_provider_weight: f64,
    total_memory_mb: u64,
    provider_count: usize,
}

fn build_provider_subsystem(config: &ProviderSubsystemConfig) -> SubsystemInfo {
    let is_active = config.provider.status == "active";

    SubsystemInfo {
        id: format!("{}:{}", config.subsystem_type.as_str(), config.provider.id),
        name: format!("{}: {}", config.type_name, config.provider.name),
        subsystem_type: config.subsystem_type.clone(),
        status: if is_active {
            SubsystemStatus::Running
        } else {
            SubsystemStatus::Stopped
        },
        health: HealthCheck {
            name: config.provider.name.clone(),
            status: config.provider.status.clone(),
            message: format!("{} operational", config.type_name),
            duration_ms: 0,
            details: Some(config.provider.config.clone()),
        },
        config: config.provider.config.clone(),
        metrics: SubsystemMetrics {
            cpu_percent: if is_active {
                config.process_metrics.cpu_percent as f64 * config.per_provider_weight
            } else {
                0.0
            },
            memory_mb: if is_active {
                (config.total_memory_mb as f64 * config.per_provider_weight) as u64
            } else {
                0
            },
            requests_per_sec: config.perf.total_queries as f64
                / config.perf.uptime_seconds.max(1) as f64
                / config.provider_count as f64,
            error_rate: 0.0,
            last_activity: Some(chrono::Utc::now()),
        },
    }
}

fn add_core_subsystems(
    subsystems: &mut Vec<SubsystemInfo>,
    process_metrics: &ProcessMetrics,
    perf: &PerformanceMetricsData,
    config: &Config,
    weights: &SubsystemWeights,
    total_memory_mb: u64,
    active_indexing_count: usize,
) {
    let queries_per_sec = perf.total_queries as f64 / perf.uptime_seconds.max(1) as f64;
    let error_rate = perf.failed_queries as f64 / perf.total_queries.max(1) as f64;

    // Search service
    subsystems.push(SubsystemInfo {
        id: "search".to_string(),
        name: "Search Service".to_string(),
        subsystem_type: SubsystemType::Search,
        status: SubsystemStatus::Running,
        health: HealthCheck {
            name: "search".to_string(),
            status: "healthy".to_string(),
            message: format!("{} queries processed", perf.total_queries),
            duration_ms: perf.average_response_time_ms as u64,
            details: Some(serde_json::json!({
                "avg_response_time_ms": perf.average_response_time_ms,
                "successful_queries": perf.successful_queries,
            })),
        },
        config: serde_json::json!({}),
        metrics: SubsystemMetrics {
            cpu_percent: process_metrics.cpu_percent as f64 * weights.search,
            memory_mb: (total_memory_mb as f64 * weights.search) as u64,
            requests_per_sec: queries_per_sec,
            error_rate,
            last_activity: Some(chrono::Utc::now()),
        },
    });

    // Indexing service
    let is_indexing = active_indexing_count > 0;
    subsystems.push(SubsystemInfo {
        id: "indexing".to_string(),
        name: "Indexing Service".to_string(),
        subsystem_type: SubsystemType::Indexing,
        status: SubsystemStatus::Running,
        health: HealthCheck {
            name: "indexing".to_string(),
            status: if is_indexing { "busy" } else { "healthy" }.to_string(),
            message: if is_indexing {
                format!("{} indexing operations in progress", active_indexing_count)
            } else {
                "Indexing service ready".to_string()
            },
            duration_ms: 0,
            details: None,
        },
        config: serde_json::json!({}),
        metrics: SubsystemMetrics {
            cpu_percent: process_metrics.cpu_percent as f64 * weights.indexing,
            memory_mb: (total_memory_mb as f64 * weights.indexing) as u64,
            requests_per_sec: if is_indexing {
                active_indexing_count as f64
            } else {
                0.0
            },
            error_rate: 0.0,
            last_activity: Some(chrono::Utc::now()),
        },
    });

    // Cache manager
    let cache_enabled = config.cache.enabled;
    subsystems.push(SubsystemInfo {
        id: "cache".to_string(),
        name: "Cache Manager".to_string(),
        subsystem_type: SubsystemType::Cache,
        status: if cache_enabled {
            SubsystemStatus::Running
        } else {
            SubsystemStatus::Stopped
        },
        health: HealthCheck {
            name: "cache".to_string(),
            status: if cache_enabled { "healthy" } else { "disabled" }.to_string(),
            message: format!("Cache hit rate: {:.1}%", perf.cache_hit_rate * 100.0),
            duration_ms: 0,
            details: Some(serde_json::json!({
                "hit_rate": perf.cache_hit_rate,
                "enabled": cache_enabled,
            })),
        },
        config: serde_json::json!({
            "enabled": cache_enabled,
            "backend": match &config.cache.backend {
                CacheBackendConfig::Local { max_entries, .. } => {
                    serde_json::json!({ "type": "local", "max_entries": max_entries })
                }
                CacheBackendConfig::Redis { url, .. } => {
                    serde_json::json!({ "type": "redis", "url": url })
                }
            },
        }),
        metrics: SubsystemMetrics {
            cpu_percent: if cache_enabled {
                process_metrics.cpu_percent as f64 * weights.cache
            } else {
                0.0
            },
            memory_mb: if cache_enabled {
                (total_memory_mb as f64 * weights.cache) as u64
            } else {
                0
            },
            requests_per_sec: queries_per_sec * perf.cache_hit_rate,
            error_rate: 0.0,
            last_activity: Some(chrono::Utc::now()),
        },
    });

    // HTTP transport
    subsystems.push(SubsystemInfo {
        id: "http_transport".to_string(),
        name: "HTTP Transport".to_string(),
        subsystem_type: SubsystemType::HttpTransport,
        status: SubsystemStatus::Running,
        health: HealthCheck {
            name: "http_transport".to_string(),
            status: "healthy".to_string(),
            message: format!("{} active connections", perf.active_connections),
            duration_ms: 0,
            details: Some(serde_json::json!({
                "active_connections": perf.active_connections,
                "port": config.metrics.port,
            })),
        },
        config: serde_json::json!({
            "port": config.metrics.port,
        }),
        metrics: SubsystemMetrics {
            cpu_percent: process_metrics.cpu_percent as f64 * weights.http,
            memory_mb: (total_memory_mb as f64 * weights.http) as u64,
            requests_per_sec: queries_per_sec,
            error_rate: 0.0,
            last_activity: Some(chrono::Utc::now()),
        },
    });
}

impl SubsystemType {
    fn as_str(&self) -> &'static str {
        match self {
            SubsystemType::Embedding => "embedding",
            SubsystemType::VectorStore => "vector_store",
            SubsystemType::Search => "search",
            SubsystemType::Indexing => "indexing",
            SubsystemType::Cache => "cache",
            SubsystemType::HttpTransport => "http_transport",
            SubsystemType::Metrics => "metrics",
            SubsystemType::Daemon => "daemon",
        }
    }
}

// ============================================================================
// Signal dispatch helpers
// ============================================================================

use crate::application::admin::types::{SignalResult, SubsystemSignal};
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};

/// Get the string name for a signal
pub fn signal_name(signal: &SubsystemSignal) -> &'static str {
    match signal {
        SubsystemSignal::Restart => "restart",
        SubsystemSignal::Reload => "reload",
        SubsystemSignal::Pause => "pause",
        SubsystemSignal::Resume => "resume",
        SubsystemSignal::Configure(_) => "configure",
    }
}

/// Parse subsystem ID into (type, id) tuple
/// Format: "type:id" or just "id"
pub fn parse_subsystem_id(subsystem_id: &str) -> (&str, &str) {
    if subsystem_id.contains(':') {
        let parts: Vec<&str> = subsystem_id.splitn(2, ':').collect();
        (parts[0], parts.get(1).copied().unwrap_or(""))
    } else {
        ("", subsystem_id)
    }
}

/// Dispatch a signal to a subsystem via the event bus
pub async fn dispatch_subsystem_signal(
    event_bus: &SharedEventBusProvider,
    subsystem_id: &str,
    signal: SubsystemSignal,
) -> SignalResult {
    let name = signal_name(&signal);
    let (subsystem_type, provider_id) = parse_subsystem_id(subsystem_id);

    // Dispatch appropriate event based on signal type and subsystem
    match signal {
        SubsystemSignal::Restart => {
            if subsystem_type == "embedding" || subsystem_type == "vector_store" {
                let _ = event_bus
                    .publish(SystemEvent::ProviderRestart {
                        provider_type: subsystem_type.to_string(),
                        provider_id: provider_id.to_string(),
                    })
                    .await;
            } else if subsystem_id == "cache" {
                let _ = event_bus
                    .publish(SystemEvent::CacheClear { namespace: None })
                    .await;
            } else if subsystem_id == "indexing" {
                let _ = event_bus
                    .publish(SystemEvent::IndexRebuild { collection: None })
                    .await;
            }
        }
        SubsystemSignal::Reload => {
            let _ = event_bus.publish(SystemEvent::Reload).await;
        }
        SubsystemSignal::Configure(config) => {
            if subsystem_type == "embedding" || subsystem_type == "vector_store" {
                let _ = event_bus
                    .publish(SystemEvent::ProviderReconfigure {
                        provider_type: subsystem_type.to_string(),
                        config,
                    })
                    .await;
            }
        }
        SubsystemSignal::Pause | SubsystemSignal::Resume => {
            // Pause/Resume not yet implemented for all subsystems
            tracing::warn!(
                "[ADMIN] Pause/Resume not implemented for subsystem: {}",
                subsystem_id
            );
        }
    }

    tracing::info!("[ADMIN] Sent {} signal to subsystem {}", name, subsystem_id);

    SignalResult {
        success: true,
        subsystem_id: subsystem_id.to_string(),
        signal: name.to_string(),
        message: format!("Signal '{}' sent to '{}'", name, subsystem_id),
    }
}
