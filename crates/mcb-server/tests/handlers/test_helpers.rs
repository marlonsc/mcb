use mcb_domain::value_objects::SessionId;
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_infrastructure::di::modules::domain_services::DomainServicesContainer;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};

/// Helper to create a base MemoryArgs with common defaults
pub(crate) fn create_base_memory_args(
    action: MemoryAction,
    resource: MemoryResource,
    data: Option<serde_json::Value>,
    ids: Option<Vec<String>>,
    session_id: Option<String>,
) -> MemoryArgs {
    MemoryArgs {
        action,
        org_id: None,
        resource,
        project_id: None,
        data,
        ids,
        repo_id: None,
        session_id: session_id.map(|id| SessionId::from_string(&id)),
        parent_session_id: None,
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    }
}

pub(crate) async fn create_real_domain_services()
-> Option<(DomainServicesContainer, tempfile::TempDir)> {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = ConfigLoader::new().load().expect("load config");
    config.providers.database.configs.insert(
        "default".to_string(),
        mcb_infrastructure::config::DatabaseConfig {
            provider: "sqlite".to_string(),
            path: Some(temp_dir.path().join("test.db")),
        },
    );
    config.providers.embedding.cache_dir = Some(temp_dir.path().join("fastembed-cache"));
    let ctx = match init_app(config).await {
        Ok(ctx) => ctx,
        Err(e)
            if e.to_string().contains("model.onnx")
                || e.to_string().contains("Failed to initialize FastEmbed") =>
        {
            eprintln!("Skipping: embedding model unavailable in offline env: {e}");
            return None;
        }
        Err(e) => panic!("init app context: {e}"),
    };
    let services = ctx
        .build_domain_services()
        .await
        .expect("build domain services");
    Some((services, temp_dir))
}
