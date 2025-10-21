use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use alloy::{
    providers::Provider,
    rpc::types::eth::{Block, BlockId},
};
use async_trait::async_trait;
use tracing::error;

use crate::interface::CollectorInterface;
use crate::interface::collector::CollectorStream;

pub struct PollFullBlockCollector {
    provider: Arc<dyn Provider>,
    interval: Duration,
    current_block: AtomicU64,
}

impl PollFullBlockCollector {
    pub fn new(provider: Arc<dyn Provider>, interval: Duration) -> Self {
        Self {
            provider,
            interval,
            current_block: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl CollectorInterface<Block> for PollFullBlockCollector {
    fn name(&self) -> &str {
        "Poll Full Block Collector"
    }

    async fn get_event_stream(&self) -> eyre::Result<CollectorStream<'_, Block>> {
        let stream = async_stream::stream! {
            loop {
                match self.provider.get_block(BlockId::latest()).full().await {
                    Ok(Some(block)) => {
                        let current_block = block.header.number;

                        let old_block = self
                            .current_block
                            .fetch_max(current_block, Ordering::Relaxed);

                        if old_block < current_block {
                            yield block;
                        }
                    }
                    Ok(None) => {
                        error!("latest block not found");
                    }
                    Err(e) => {
                        error!("fail to get latest block: {e:#}")
                    }
                };

                tokio::time::sleep(self.interval).await;
            }
        };

        Ok(Box::pin(stream))
    }
}
