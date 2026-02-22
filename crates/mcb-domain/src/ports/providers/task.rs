use futures::future::BoxFuture;

use crate::error::Result;

#[allow(missing_docs, dead_code, clippy::missing_errors_doc)]
pub trait TaskRunnerProvider: Send + Sync {
    fn spawn(&self, task: BoxFuture<'static, ()>) -> Result<()>;
}
