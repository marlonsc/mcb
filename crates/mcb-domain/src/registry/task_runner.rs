#![allow(missing_docs)]

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TaskRunnerProviderConfig {
    pub provider: String,
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(TaskRunnerProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::TaskRunnerProvider,
    config_type: TaskRunnerProviderConfig,
    entry_type: TaskRunnerProviderEntry,
    slice_name: TASK_RUNNER_PROVIDERS,
    resolve_fn: resolve_task_runner_provider,
    list_fn: list_task_runner_providers
);
