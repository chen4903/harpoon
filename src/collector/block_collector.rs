use alloy::{providers::Provider, rpc::types::Header};
use async_trait::async_trait;
use std::sync::Arc;

use crate::interface::{CollectorInterface, collector::CollectorStream};

pub struct BlockCollector {
    provider: Arc<dyn Provider>,
}

impl BlockCollector {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl CollectorInterface<Header> for BlockCollector {
    fn name(&self) -> &str {
        "Block Collector"
    }

    async fn get_event_stream(&self) -> eyre::Result<CollectorStream<'_, Header>> {
        let stream = self.provider.subscribe_blocks().await?;

        Ok(Box::pin(stream.into_stream()))
    }
}
