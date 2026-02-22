use futures::future::BoxFuture;
use mcb_domain::error::Result;
use mcb_domain::ports::TaskRunnerProvider;
use mcb_domain::registry::task_runner::{
    TASK_RUNNER_PROVIDERS, TaskRunnerProviderConfig, TaskRunnerProviderEntry,
};

fn create_tokio_task_runner_provider(
    _config: &TaskRunnerProviderConfig,
) -> std::result::Result<std::sync::Arc<dyn TaskRunnerProvider>, String> {
    Ok(std::sync::Arc::new(TokioTaskRunnerProvider::new()))
}

#[linkme::distributed_slice(TASK_RUNNER_PROVIDERS)]
static TOKIO_TASK_RUNNER_PROVIDER: TaskRunnerProviderEntry = TaskRunnerProviderEntry {
    name: "tokio",
    description: "Tokio task runner provider",
    build: create_tokio_task_runner_provider,
};

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct TokioTaskRunnerProvider;

#[allow(missing_docs)]
impl TokioTaskRunnerProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl TaskRunnerProvider for TokioTaskRunnerProvider {
    fn spawn(&self, task: BoxFuture<'static, ()>) -> Result<()> {
        tokio::spawn(task);
        Ok(())
    }
}
