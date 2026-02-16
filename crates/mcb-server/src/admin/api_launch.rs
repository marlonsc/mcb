use std::future::Future;

use super::api::AdminApi;
use super::routes::admin_rocket;

fn rocket_error(
    phase: &str,
    err: impl std::fmt::Display,
) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(format!(
        "Rocket {phase} failed: {err}"
    )))
}

impl AdminApi {
    /// Start the admin API server.
    ///
    /// # Errors
    /// Returns an error when Rocket fails to launch.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket =
            admin_rocket(self.state, self.auth_config, self.browse_state).configure(rocket_config);

        rocket
            .launch()
            .await
            .map_err(|e| rocket_error("launch", e))?;
        Ok(())
    }

    /// Start the admin API server with graceful shutdown.
    ///
    /// # Errors
    /// Returns an error when Rocket fails to ignite or launch.
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket = admin_rocket(self.state, self.auth_config, self.browse_state)
            .configure(rocket_config)
            .ignite()
            .await
            .map_err(|e| rocket_error("ignite", e))?;

        let shutdown_handle = rocket.shutdown();
        tokio::spawn(async move {
            shutdown_signal.await;
            shutdown_handle.notify();
        });

        rocket
            .launch()
            .await
            .map_err(|e| rocket_error("launch", e))?;
        Ok(())
    }
}
