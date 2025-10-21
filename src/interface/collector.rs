use std::pin::Pin;

use async_trait::async_trait;
use eyre::Result;
use futures::Stream;

pub(crate) type CollectorStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

#[async_trait]
pub trait CollectorInterface<E>: Send + Sync {
    fn name(&self) -> &str {
        "Unnamed"
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E>>;
}
