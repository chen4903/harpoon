use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;

use crate::interface::action_submitter::ActionSubmitterInterface;

#[async_trait]
pub trait StrategyInterface<E, A>: Send + Sync
where
    E: Send + Sync + Clone + 'static,
    A: Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn sync_state(&mut self, _submitter: Arc<dyn ActionSubmitterInterface<A>>) -> Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: E, submitter: Arc<dyn ActionSubmitterInterface<A>>);
}
