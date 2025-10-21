use std::pin::Pin;

use async_trait::async_trait;
use eyre::Result;
use futures::Stream;

pub type CollectorStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

#[async_trait]
pub trait Collector<E>: Send + Sync {
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E>>;
}

pub trait ActionSubmitter<A>: Send + Sync
where
    A: Send + Sync + Clone + 'static,
{
    fn submit(&self, action: A);
}
