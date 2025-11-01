use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;

use crate::IActionSubmitter;

#[async_trait]
pub trait IStrategy<E, A>: Send + Sync
where
    E: Send + Sync + Clone + 'static,
    A: Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn sync_state(&mut self, _submitter: Arc<dyn IActionSubmitter<A>>) -> Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: E, submitter: Arc<dyn IActionSubmitter<A>>);
}
