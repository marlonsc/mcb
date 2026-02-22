use futures::future::BoxFuture;

use crate::error::Result;

/// Port for spawning background tasks (used by DI; implementations in mcb-providers).
pub trait TaskRunnerProvider: Send + Sync {
    /// Spawns a background task.
    ///
    /// # Errors
    /// Returns an error if the task could not be spawned.
    fn spawn(&self, task: BoxFuture<'static, ()>) -> Result<()>;
}
