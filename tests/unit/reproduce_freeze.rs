#[cfg(test)]
mod reproduction_test {
    use mcp_context_browser::infrastructure::cache::{
        CacheBackendConfig, CacheConfig, CacheNamespacesConfig, create_cache_provider,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_concurrent_access_freeze() -> Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig {
            enabled: true,
            backend: CacheBackendConfig::Local {
                max_entries: 1000,
                default_ttl_seconds: 60,
            },
            namespaces: CacheNamespacesConfig::default(),
        };

        let cache = Arc::new(create_cache_provider(&config).await?);
        let mut handles = vec![];
        let ttl = Duration::from_secs(60);

        // Spawn 100 concurrent readers/writers
        for _i in 0..100 {
            let c = cache.clone();
            handles.push(tokio::spawn(async move {
                for j in 0..1000 {
                    let key = format!("key-{}", j % 100);
                    if j % 10 == 0 {
                        let value = "value".to_string().into_bytes();
                        let _ = c.set("test", &key, value, ttl).await;
                    } else {
                        let _ = c.get("test", &key).await;
                    }
                }
            }));
        }

        // Should finish quickly, but if it locks up, timeout will catch it
        let result = timeout(Duration::from_secs(5), async {
            for h in handles {
                h.await?;
            }
            Ok::<(), tokio::task::JoinError>(())
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err("Test timed out - likely deadlock or extreme contention".into()),
        }
    }
}
