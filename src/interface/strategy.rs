use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;

pub trait ActionSubmitter<A>: Send + Sync
where
    A: Send + Sync + Clone + 'static,
{
    fn submit(&self, action: A);
}

#[async_trait]
pub trait Strategy<E, A>: Send + Sync
where
    E: Send + Sync + Clone + 'static,
    A: Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn sync_state(&mut self, _submitter: Arc<dyn ActionSubmitter<A>>) -> Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: E, submitter: Arc<dyn ActionSubmitter<A>>);
}
