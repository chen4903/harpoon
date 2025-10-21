use std::sync::Arc;
use std::time::Duration;

use alloy::{providers::Provider, rpc::types::eth::Block};
use async_trait::async_trait;
use futures::StreamExt;
use tracing::{error, warn};

use crate::{CollectorInterface, CollectorStream};

pub struct FullBlockCollector {
    provider: Arc<dyn Provider>,
    retry_interval: Duration,
}

impl FullBlockCollector {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self::new_with_config(provider, Duration::from_millis(50))
    }

    /// Create a new `FullBlockCollector` with a custom retry interval. A retry will happen when the client returns
    /// "header not found"
    pub fn new_with_config(provider: Arc<dyn Provider>, retry_interval: Duration) -> Self {
        Self {
            provider,
            retry_interval,
        }
    }
}

#[async_trait]
impl CollectorInterface<Block> for FullBlockCollector {
    fn name(&self) -> &str {
        "Full Block Collector"
    }

    async fn get_event_stream(&self) -> eyre::Result<CollectorStream<'_, Block>> {
        let mut stream = self.provider.subscribe_blocks().await?.into_stream();

        let mut attempts = 0;

        let stream = async_stream::stream! {
            while let Some(block) = stream.next().await {
                let block_number = block.number;

                loop {
                    match self.provider.get_block_by_number(block_number.into()).full().await {
                        Ok(Some(block)) => {
                            yield block;
                        }
                        Ok(None) => {
                            if attempts % 5 == 0 {
                                warn!("block not found yet: {}", block_number);
                            } else {
                                error!("block not found yet: {}", block_number);
                            }

                            attempts += 1;
                            tokio::time::sleep(self.retry_interval).await;
                            continue;
                        }
                        Err(e) => {
                            error!("fail to get full block: {:#}, block number: {}", e, block_number);
                        }
                    };

                    break;
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
